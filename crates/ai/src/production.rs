//! City decisions: what idle cities produce, occasional gold purchases.

use crate::analysis::{attack_power, defense_power};
use crate::composition;
use crate::purchase_defense::purchased_defender;
use crate::scripted::{Planner, tag};
use legatum_core::data;
use legatum_core::view::{CityView, ProductionOption, UnitView};
use legatum_core::{
    AiDifficulty, BuildingKind, CityBuildFocus, CityId, Faction, Hex, Order, PlayerView,
    ProductionItem, UnitKind,
};

/// Diagnostics only (`ScriptedBrain::production_diagnosis`): what an idle city weighed and
/// chose. Recording never changes a decision.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ProductionTrace {
    pub city: legatum_core::CityId,
    pub chosen: Option<ProductionItem>,
    /// Every scored option: (item, value before the time discount, discounted score).
    pub options: Vec<(ProductionItem, i32, i32)>,
    pub settler: Option<SettlerGates>,
    pub gates: Vec<(ProductionItem, String)>,
    pub early_gate: Option<&'static str>,
}

#[cfg(test)]
mod siege_deadline_tests {
    use super::*;
    use crate::Knobs;
    use legatum_core::{GameState, PlayerId, player_view};

    #[test]
    fn gergovia_t29_starts_the_timely_soldier_instead_of_the_late_ranger() {
        let state: GameState = serde_json::from_reader(flate2::read::GzDecoder::new(
            &include_bytes!("../tests/fixtures/gergovia-t29.json.gz")[..],
        ))
        .unwrap();
        let view = player_view(&state, PlayerId(5));
        let city = view.cities.iter().find(|c| c.name == "Gergovia").unwrap();
        let ranger = ProductionItem::Unit {
            unit: UnitKind::Archer,
        };
        let soldier = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        let detail = city.detail.as_ref().unwrap();
        let ranger_turns = detail
            .options
            .iter()
            .find(|o| o.item == ranger)
            .unwrap()
            .turns;
        let soldier_turns = detail
            .options
            .iter()
            .find(|o| o.item == soldier)
            .unwrap()
            .turns;
        for (gene, expected) in [(0, ranger), (1, soldier)] {
            let mut p = Planner::new(
                &view,
                Knobs::for_faction(view.faction_of(view.me)),
                view.faction_of(view.me),
                7,
            );
            p.difficulty = AiDifficulty::Hard;
            p.decisions.siege_survival_clock = gene;
            p.decisions.production_threat_agreement = 1;
            p.decisions.walled_defense_percent = 100;
            p.decisions.keep_walls = 1;
            p.orders.push(Order::SetProduction {
                city: city.id,
                item: ranger,
            });
            if gene == 1 {
                let deadline = p.defense_risk(city, None).production_deadline().unwrap();
                assert!(soldier_turns < deadline && ranger_turns >= deadline);
            }
            p.enforce_siege_deadlines();
            assert_eq!(
                p.orders,
                vec![Order::SetProduction {
                    city: city.id,
                    item: expected
                }]
            );
        }
    }
}

#[cfg(test)]
mod wonder_tile_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::view::WonderPreview;

    #[test]
    fn chooses_lower_yield_and_safer_wonder_site() {
        let view = ViewBuilder::new(24, 24)
            .city(1, ME, off(6, 6), 4)
            .city(2, ENEMY, off(18, 18), 4)
            .build();
        let planner = Planner::new(&view, Knobs::for_faction(Faction::India), Faction::India, 7);
        let mut option = view.cities[0].detail.as_ref().unwrap().options[0].clone();
        let safe = off(5, 6);
        option.item = ProductionItem::Wonder {
            building: BuildingKind::GreatLibrary,
            tile: safe,
        };
        option.wonder = Some(WonderPreview {
            tile: safe,
            lost: legatum_core::Yields::new(0, 1, 0, 0),
        });
        let cheap = planner.wonder_tile_penalty(&option);
        option.wonder.as_mut().unwrap().lost = legatum_core::Yields::new(0, 4, 0, 0);
        assert!(cheap < planner.wonder_tile_penalty(&option));
        option.wonder.as_mut().unwrap().lost = legatum_core::Yields::new(0, 1, 0, 0);
        option.wonder.as_mut().unwrap().tile = off(17, 18);
        assert!(cheap < planner.wonder_tile_penalty(&option));
    }
}

#[cfg(test)]
mod purchase_scorer_tests {
    use super::*;
    use crate::{
        Knobs,
        fixture::{ENEMY, ME, ViewBuilder},
    };
    use legatum_core::{Order, TechId};

    fn planner(v: &PlayerView, gene: i32) -> Planner<'_> {
        let mut p = Planner::new(v, Knobs::for_faction(v.faction_of(ME)), v.faction_of(ME), 7);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.hard_purchase_scorer = gene;
        p.decisions.buy_hold = 0;
        p
    }

    fn quoted(mut v: PlayerView, choices: &[UnitKind]) -> PlayerView {
        for city in v.cities.iter_mut().filter(|c| c.owner == ME) {
            let detail = city.detail.as_mut().unwrap();
            detail.options.retain(
                |o| matches!(o.item, ProductionItem::Unit { unit } if choices.contains(&unit)),
            );
            for option in &mut detail.options {
                option.buy_cost = Some(option.cost * 2);
            }
        }
        v
    }

    fn first_buy(p: &Planner<'_>) -> Option<UnitKind> {
        p.orders.iter().find_map(|order| match order {
            Order::BuyItem {
                item: ProductionItem::Unit { unit },
                ..
            } => Some(*unit),
            _ => None,
        })
    }

    #[test]
    fn generation_floor_uses_only_the_immediate_predecessor_for_missing_supply() {
        let mut v = quoted(
            ViewBuilder::new(24, 16)
                .known_techs(&TechId::ALL)
                .gold(800)
                .city(1, ME, Hex::new(6, 6), 4)
                .build(),
            &[
                UnitKind::Spearman,
                UnitKind::Phalangite,
                UnitKind::Pikeman,
                UnitKind::Musketman,
            ],
        );
        let options = &mut v.cities[0].detail.as_mut().unwrap().options;
        options
            .iter_mut()
            .find(|o| {
                o.item
                    == ProductionItem::Unit {
                        unit: UnitKind::Musketman,
                    }
            })
            .unwrap()
            .buy_block = Some(legatum_core::economy::BuyBlock::Resource);
        let mut p = planner(&v, 1);
        p.decisions.purchase_generation_floor = 1;
        assert_eq!(
            p.purchase_generation(&v.cities[0], UnitKind::Spearman),
            Some(UnitKind::Pikeman)
        );
        p.gold_left = 800 - 2 * data::unit_stats(UnitKind::Spearman).cost;
        p.orders.push(Order::BuyItem {
            city: CityId(1),
            item: ProductionItem::Unit {
                unit: UnitKind::Spearman,
            },
        });
        p.enforce_purchase_generation();
        assert_eq!(first_buy(&p), Some(UnitKind::Pikeman));

        let options = &mut v.cities[0].detail.as_mut().unwrap().options;
        options
            .iter_mut()
            .find(|o| {
                o.item
                    == ProductionItem::Unit {
                        unit: UnitKind::Pikeman,
                    }
            })
            .unwrap()
            .buy_block = Some(legatum_core::economy::BuyBlock::Resource);
        let mut p = planner(&v, 1);
        p.decisions.purchase_generation_floor = 1;
        p.gold_left = 800 - 2 * data::unit_stats(UnitKind::Spearman).cost;
        p.orders.push(Order::BuyItem {
            city: CityId(1),
            item: ProductionItem::Unit {
                unit: UnitKind::Spearman,
            },
        });
        p.enforce_purchase_generation();
        assert_eq!(first_buy(&p), None);
    }

    #[test]
    fn generation_floor_saves_gold_when_newest_is_trainable_but_unaffordable() {
        let v = quoted(
            ViewBuilder::new(24, 16)
                .known_techs(&TechId::ALL)
                .gold(200)
                .city(1, ME, Hex::new(6, 6), 4)
                .build(),
            &[UnitKind::Spearman, UnitKind::Pikeman, UnitKind::Musketman],
        );
        let mut p = planner(&v, 1);
        p.decisions.purchase_generation_floor = 1;
        p.gold_left = 200 - 2 * data::unit_stats(UnitKind::Spearman).cost;
        p.orders.push(Order::BuyItem {
            city: CityId(1),
            item: ProductionItem::Unit {
                unit: UnitKind::Spearman,
            },
        });
        p.enforce_purchase_generation();
        assert_eq!(first_buy(&p), None);
        assert_eq!(p.gold_left, 200);
    }

    #[test]
    fn athens_t131_one_mobile_tank_beats_two_obsolete_bodies() {
        let v = ViewBuilder::new(24, 16)
            .known_techs(&TechId::ALL)
            .gold(10_000)
            .city(1, ME, Hex::new(6, 6), 4)
            .unit(2, ENEMY, UnitKind::Tank, Hex::new(12, 6))
            .build();
        let mut p = planner(&v, 1);
        p.decisions.purchase_generation_floor = 0;
        let quote = |unit| 2 * data::unit_stats(unit).cost;
        assert!(
            p.military_purchase_score(UnitKind::Tank, quote(UnitKind::Tank), v.cities[0].pos)
                > p.military_purchase_score(
                    UnitKind::Musketman,
                    quote(UnitKind::Musketman),
                    v.cities[0].pos
                )
        );
        assert!(
            p.military_purchase_score(UnitKind::Tank, quote(UnitKind::Tank), v.cities[0].pos)
                > p.military_purchase_score(
                    UnitKind::Spearman,
                    quote(UnitKind::Spearman),
                    v.cities[0].pos
                )
        );
    }

    #[test]
    fn athens_t131_keeps_gold_for_trainable_landship_instead_of_soldier() {
        let mut v = quoted(
            ViewBuilder::new(24, 16)
                .known_techs(&TechId::ALL)
                .gold(544)
                .city(1, ME, Hex::new(6, 6), 4)
                .unit(2, ENEMY, UnitKind::EarlyTank, Hex::new(12, 6))
                .build(),
            &[UnitKind::Spearman, UnitKind::Musketman, UnitKind::EarlyTank],
        );
        v.economy.gold_per_turn.total = 78;
        let city = &v.cities[0];
        let soldier_quote = city
            .detail
            .as_ref()
            .unwrap()
            .options
            .iter()
            .find(|o| {
                o.item
                    == ProductionItem::Unit {
                        unit: UnitKind::Spearman,
                    }
            })
            .unwrap()
            .buy_cost
            .unwrap();
        assert!(planner(&v, 0).military_purchase_viable(city, UnitKind::Spearman, soldier_quote));
        assert!(!planner(&v, 1).military_purchase_viable(city, UnitKind::Spearman, soldier_quote));
    }

    #[test]
    fn strong_city_shield_lets_an_old_garrison_meet_an_imminent_front() {
        let mut v = quoted(
            ViewBuilder::new(24, 16)
                .known_techs(&TechId::ALL)
                .gold(189)
                .city(1, ME, Hex::new(6, 6), 4)
                .unit(50, ENEMY, UnitKind::Artillery, Hex::new(10, 6))
                .unit(51, ENEMY, UnitKind::Spearman, Hex::new(10, 5))
                .unit(52, ENEMY, UnitKind::Spearman, Hex::new(11, 6))
                .build(),
            &[UnitKind::Spearman, UnitKind::Rifleman],
        );
        v.economy.gold_per_turn.total = 77;
        v.cities[0].walls = true;
        v.cities[0].star_fort = true;
        v.cities[0].strength = 93;
        v.cities[0]
            .detail
            .as_mut()
            .unwrap()
            .options
            .iter_mut()
            .find(|o| {
                o.item
                    == ProductionItem::Unit {
                        unit: UnitKind::Rifleman,
                    }
            })
            .unwrap()
            .buy_cost = Some(522);
        let city = &v.cities[0];
        let soldier_quote = city
            .detail
            .as_ref()
            .unwrap()
            .options
            .iter()
            .find(|o| {
                o.item
                    == ProductionItem::Unit {
                        unit: UnitKind::Spearman,
                    }
            })
            .unwrap()
            .buy_cost
            .unwrap();
        let mut p = planner(&v, 1);
        // This fixture tests the former one-turn shield allowance in the purchase scorer.
        p.decisions.siege_survival_clock = 0;
        let rifle_quote = city
            .detail
            .as_ref()
            .unwrap()
            .options
            .iter()
            .find(|o| {
                o.item
                    == ProductionItem::Unit {
                        unit: UnitKind::Rifleman,
                    }
            })
            .unwrap()
            .buy_cost
            .unwrap();
        assert!(!p.defense_risk(city, None).falls);
        assert!(rifle_quote <= v.economy.gold_per_turn.total * 5 + p.gold_left);
        assert!(
            p.military_purchase_score(UnitKind::Rifleman, rifle_quote, city.pos)
                >= p.military_purchase_score(UnitKind::Spearman, soldier_quote, city.pos) * 2
        );
        assert!(
            p.door_fronts()
                .iter()
                .any(|f| f.city == city.id && f.arrival <= 1 && f.gap() > 0)
        );
        assert!(p.military_purchase_viable(city, UnitKind::Spearman, soldier_quote));
    }

    #[test]
    fn athens_t101_mounted_attack_favors_surviving_melee_and_due_upgrade_is_held() {
        let mut v = ViewBuilder::new(24, 16)
            .known_techs(&[TechId::Engineering, TechId::Gunpowder])
            .researching(TechId::Rifling)
            .gold(1000)
            .city(1, ME, Hex::new(6, 6), 4)
            .unit(1, ME, UnitKind::Musketman, Hex::new(6, 6))
            .unit(2, ENEMY, UnitKind::Cuirassier, Hex::new(10, 6))
            .build();
        v.research.turns_left = Some(2);
        let p = planner(&v, 1);
        let quote = |unit| 2 * data::unit_stats(unit).cost;
        assert!(
            p.military_purchase_score(
                UnitKind::Musketman,
                quote(UnitKind::Musketman),
                v.cities[0].pos
            ) > p.military_purchase_score(
                UnitKind::CompositeArcher,
                quote(UnitKind::CompositeArcher),
                v.cities[0].pos
            )
        );
        assert!(p.military_upgrade_hold(&v.cities[0]));
        assert!(!planner(&v, 0).military_upgrade_hold(&v.cities[0]));
    }

    #[test]
    fn celts_t115_era_defender_outscores_spearman_against_tank() {
        let v = ViewBuilder::new(24, 16)
            .faction(Faction::Celts)
            .known_techs(&TechId::ALL)
            .gold(10_000)
            .city(1, ME, Hex::new(6, 6), 4)
            .unit(2, ENEMY, UnitKind::Tank, Hex::new(9, 6))
            .build();
        let mut p = planner(&v, 1);
        p.decisions.purchase_generation_floor = 0;
        let quote = |unit| 2 * data::unit_stats(unit).cost;
        assert!(
            p.military_purchase_score(UnitKind::Tank, quote(UnitKind::Tank), v.cities[0].pos)
                > p.military_purchase_score(
                    UnitKind::Spearman,
                    quote(UnitKind::Spearman),
                    v.cities[0].pos
                )
        );
    }

    #[test]
    fn celts_t115_door_saves_when_only_obsolete_unit_is_affordable() {
        let mut v = quoted(
            ViewBuilder::new(24, 16)
                .faction(Faction::Celts)
                .known_techs(&TechId::ALL)
                .gold(166)
                .city(1, ME, Hex::new(6, 6), 4)
                .unit(2, ENEMY, UnitKind::Rifleman, Hex::new(1, 5))
                .unit(3, ENEMY, UnitKind::Cavalry, Hex::new(2, 4))
                .unit(4, ENEMY, UnitKind::Cavalry, Hex::new(2, 5))
                .build(),
            &[UnitKind::Spearman, UnitKind::Pikeman],
        );
        v.economy.gold_per_turn.total = 36;
        let mut old = planner(&v, 0);
        old.plan_door_defense();
        let mut new = planner(&v, 1);
        new.plan_door_defense();
        assert_eq!(
            first_buy(&old),
            Some(UnitKind::Spearman),
            "{:?}",
            old.orders
        );
        assert_eq!(first_buy(&new), None, "{:?}", new.orders);
    }

    #[test]
    fn athens_t101_buys_a_guard_when_capture_precedes_the_due_upgrade() {
        let mut v = quoted(
            ViewBuilder::new(24, 16)
                .known_techs(&[TechId::Engineering, TechId::Gunpowder])
                .researching(TechId::Rifling)
                .gold(684)
                .city(1, ME, Hex::new(6, 6), 4)
                .unit(1, ME, UnitKind::Musketman, Hex::new(4, 6))
                .unit(2, ENEMY, UnitKind::Cavalry, Hex::new(9, 6))
                .unit(3, ENEMY, UnitKind::Cavalry, Hex::new(10, 6))
                .build(),
            &[
                UnitKind::Spearman,
                UnitKind::CompositeArcher,
                UnitKind::Musketman,
            ],
        );
        v.research.turns_left = Some(2);
        let mut old = planner(&v, 0);
        old.plan_door_defense();
        let mut new = planner(&v, 1);
        new.decisions.siege_survival_clock = 1;
        new.decisions.production_threat_agreement = 1;
        new.decisions.walled_defense_percent = 0;
        let risk = new.defense_risk(&v.cities[0], None);
        assert!(
            risk.turns_to_fall.is_some_and(|turns| turns <= 2),
            "{risk:?}"
        );
        new.plan_door_defense();
        assert!(first_buy(&old).is_some(), "{:?}", old.orders);
        assert_eq!(
            first_buy(&new),
            Some(UnitKind::Musketman),
            "{:?}",
            new.orders
        );
    }
}

/// Diagnostics for production overrides, evaluated only when an order trace is requested.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ProductionDefenseTrace {
    pub city: legatum_core::CityId,
    pub threat: i32,
    pub local_defense: i32,
    pub falls: bool,
    pub exposure: u8,
    pub early_warning: i32,
}

/// Each condition the routine Settler valuation checks, and the resulting value.
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct SettlerGates {
    pub threatened: bool,
    pub pop_ok: bool,
    pub want: bool,
    pub time: bool,
    pub room: bool,
    pub developed: bool,
    pub army_ok: bool,
    pub sites_ok: bool,
    pub pipeline_ok: bool,
    pub value: i32,
}

/// Gene `aztecs_playbook` bits (see `params.rs`).
pub(crate) const AZTEC_WALLS: i32 = 1;
pub(crate) const AZTEC_CLOSER: i32 = 2;
pub(crate) const AZTEC_LEAN_ARMY: i32 = 4;
/// Field units beyond one per city an Aztec war wants under `AZTEC_LEAN_ARMY` (the shared
/// `army_war_extra` is 1 since 2026-11-06, 5 before).
const AZTEC_WAR_EXTRA: i32 = 1;

/// Production value of a troop the garrison floor still misses (one per city, Hard plus one),
/// and under gene `swiss_playbook` of the Redoubt that replaces it.
const GARRISON_FLOOR_VALUE: i32 = 2200;
/// A capital a known rival unit can reach (`capital_pressed`): its garrison and Walls outrank
/// every other city's build, because its fall ends the capitals victory race for us.
const CAPITAL_PRESSED_VALUE: i32 = 4000;

fn item_is_settler(item: Option<ProductionItem>) -> bool {
    item == Some(ProductionItem::Unit {
        unit: UnitKind::Settler,
    })
}

/// A city's base worth, in the AI's half Production points: what a Settler cost before the 2026-10-22
/// retune (150 listed, 75 here). The Settler's price fell to 70 so the opening holds; the worth of a
/// city the AI weighs against its units did not.
const CITY_KEEP_BASE: i32 = 75;

/// A city's worth when weighing what to risk for it, in Production: a Settler's base cost
/// plus its grown population, more for a capital.
/// `city_keep_value` of a city founded now with `population` (not a capital).
pub(crate) fn new_city_keep_value(population: i32) -> i32 {
    CITY_KEEP_BASE + 20 * population
}

pub(crate) fn city_keep_value(city: &CityView) -> i32 {
    CITY_KEEP_BASE + 20 * city.population + if city.is_capital { 40 } else { 0 }
}

/// Empire-wide counts, updated as idle cities pick items so later cities see earlier picks.
#[derive(Clone, Copy)]
struct Tally {
    army: i32,
    rangers: i32,
    scouts: i32,
    /// Heirs already held, building, or selected earlier in this planning pass.
    heir_pipeline: i32,
    /// Settlers alive plus settlers being produced.
    settler_pipeline: i32,
    /// Of those in production, the ones still `settler_slow_turns` or more from finishing.
    slow_settlers: i32,
    relay_started: bool,
    /// Distinct good sites available (capped).
    sites: i32,
}

impl Tally {
    fn add(&mut self, item: ProductionItem) {
        match item {
            ProductionItem::Unit {
                unit: UnitKind::Settler,
            } => self.settler_pipeline += 1,
            ProductionItem::Unit {
                unit: UnitKind::Scout,
            } => self.scouts += 1,
            ProductionItem::Unit {
                unit: UnitKind::HeirToTheThrone,
            } => self.heir_pipeline += 1,
            ProductionItem::Unit { unit } if composition::is_ranged_line(unit) => {
                self.army += 1;
                self.rangers += 1;
            }
            item if is_military_unit(item) => self.army += 1,
            item if legatum_core::data::is_great_work_item(item) => self.relay_started = true,
            _ => {}
        }
    }
}

/// Most a class is worth for countering the enemy army we can see, in production value.
const MATCHUP_VALUE: i32 = 300;
/// Most a class gains for being under its share of our own army.
const BALANCE_VALUE: i32 = 100;
const RUSH_MOUNTED_VALUE: i32 = 100;
/// Easy and Normal's war army above the garrison: the shared `army_war_extra` before Hard went lean.
const EASIER_WAR_EXTRA: i32 = 5;

pub(crate) fn is_military_unit(item: ProductionItem) -> bool {
    matches!(item, ProductionItem::Unit { unit } if composition::is_combat(unit))
}

impl Planner<'_> {
    /// The last city-order gate: every pass that starts a defender or Walls shares the live
    /// survival clock. Production completes after battle, so finishing on the capture turn is
    /// too late. A timely cheaper defender replaces a late one; when none exists the late start
    /// is dropped. This catches threat and routine orders as well as door and emergency orders.
    pub(crate) fn enforce_siege_deadlines(&mut self) {
        if !self.hard_defense() || self.decisions.siege_survival_clock == 0 {
            return;
        }
        let mut deadlines = std::collections::BTreeMap::new();
        let mut alternatives = std::collections::BTreeMap::new();
        for &city in &self.ctx.own_cities {
            let Some(fall) = self.defense_risk(city, None).production_deadline() else {
                continue;
            };
            let Some(detail) = city.detail.as_ref() else {
                continue;
            };
            deadlines.insert(city.id, fall);
            let best = detail
                .options
                .iter()
                .filter(|o| o.turns > 0 && o.turns < fall && o.resource_blocker.is_none())
                .filter(|o| match o.item {
                    ProductionItem::Unit { unit } => composition::is_combat(unit),
                    ProductionItem::Building { building } => {
                        data::building_family(building) == BuildingKind::Walls
                    }
                    _ => false,
                })
                .max_by_key(|o| {
                    let power = match o.item {
                        ProductionItem::Unit { unit } => self.matchup_strength(unit, city.pos),
                        ProductionItem::Building { .. } => city_keep_value(city) / 4,
                        _ => 0,
                    };
                    (
                        i64::from(power.max(0)) * i64::from(fall - o.turns)
                            / i64::from(o.turns + 1),
                        std::cmp::Reverse(o.cost),
                        std::cmp::Reverse(o.item),
                    )
                })
                .map(|o| o.item);
            alternatives.insert(city.id, best);
        }
        let mut orders = Vec::with_capacity(self.orders.len());
        for order in std::mem::take(&mut self.orders) {
            let Order::SetProduction { city, item } = order else {
                orders.push(order);
                continue;
            };
            let Some(&fall) = deadlines.get(&city) else {
                orders.push(Order::SetProduction { city, item });
                continue;
            };
            let Some(home) = self.ctx.own_cities.iter().copied().find(|c| c.id == city) else {
                orders.push(Order::SetProduction { city, item });
                continue;
            };
            let defensive = match item {
                ProductionItem::Unit { unit } => composition::is_combat(unit),
                ProductionItem::Building { building } => {
                    data::building_family(building) == BuildingKind::Walls
                }
                _ => false,
            };
            let late = defensive
                && home
                    .detail
                    .as_ref()
                    .and_then(|d| d.options.iter().find(|o| o.item == item).map(|o| o.turns))
                    .is_some_and(|turns| turns >= fall);
            if !late {
                orders.push(Order::SetProduction { city, item });
            } else if let Some(Some(timely)) = alternatives.get(&city) {
                orders.push(Order::SetProduction {
                    city,
                    item: *timely,
                });
            }
        }
        self.orders = orders;
    }

    /// The legal generation for a class in this city's researched production list.
    /// A missing strategic resource permits exactly one step down the upgrade line.
    pub(crate) fn purchase_generation(&self, city: &CityView, kind: UnitKind) -> Option<UnitKind> {
        let ladder: &[UnitKind] = match composition::class(kind)? {
            legatum_core::UnitClass::Melee => &composition::MELEE_LADDER,
            legatum_core::UnitClass::Ranged => &composition::RANGED_LADDER,
            legatum_core::UnitClass::Mounted => &composition::MOUNTED_LADDER,
            legatum_core::UnitClass::Air => return Some(kind),
        };
        let options = &city.detail.as_ref()?.options;
        let (index, newest) = ladder.iter().enumerate().find_map(|(index, unit)| {
            options
                .iter()
                .find(|o| o.item == ProductionItem::Unit { unit: *unit })
                .map(|o| (index, o))
        })?;
        if newest.buy_block == Some(legatum_core::economy::BuyBlock::Resource) {
            ladder.get(index + 1).copied()
        } else {
            Some(ladder[index])
        }
    }

    /// The city has no body and falls this resolution turn without one. Only a
    /// recruit that keeps it standing through the forecast qualifies for the
    /// cheapest-body exception.
    fn purchase_rescue(&self, city: &CityView, available: i32) -> Option<(UnitKind, i32)> {
        let before = self.defense_risk(city, None);
        if self
            .ctx
            .own_units
            .iter()
            .any(|u| u.military && u.pos == city.pos)
            || before.turns_to_fall != Some(1)
        {
            return None;
        }
        city.detail
            .as_ref()?
            .options
            .iter()
            .filter_map(|o| {
                let ProductionItem::Unit { unit } = o.item else {
                    return None;
                };
                if !composition::is_combat(unit)
                    || !matches!(
                        o.buy_block,
                        None | Some(legatum_core::economy::BuyBlock::NotEnoughGold)
                    )
                {
                    return None;
                }
                let cost = o.buy_cost.filter(|cost| *cost <= available)?;
                (!self
                    .defense_risk(city, Some(purchased_defender(self.view(), city, unit)))
                    .falls)
                    .then_some((unit, cost))
            })
            .min_by_key(|(unit, cost)| (*cost, *unit))
    }

    /// All military Gold-buy passes run before this check. It also catches a pass
    /// whose own shortlist omitted the newest recruit because it was dearer.
    pub(crate) fn enforce_purchase_generation(&mut self) {
        if self.difficulty != AiDifficulty::Hard || self.decisions.purchase_generation_floor == 0 {
            return;
        }
        let mut gold = self.gold_left;
        let mut orders = self.orders.clone();
        let mut removed = std::collections::BTreeSet::new();
        for (index, order) in orders.iter_mut().enumerate() {
            let Order::BuyItem {
                city,
                item: ProductionItem::Unit { unit },
            } = order
            else {
                continue;
            };
            if !composition::is_combat(*unit) {
                continue;
            }
            let Some(home) = self.ctx.own_cities.iter().copied().find(|c| c.id == *city) else {
                continue;
            };
            let Some(detail) = home.detail.as_ref() else {
                continue;
            };
            let old_cost = detail
                .options
                .iter()
                .find(|o| o.item == ProductionItem::Unit { unit: *unit })
                .and_then(|o| o.buy_cost)
                .unwrap_or(0);
            let available = gold.saturating_add(old_cost);
            let rescue = self.purchase_rescue(home, available);
            let replacement = rescue.or_else(|| {
                let wanted = self.purchase_generation(home, *unit)?;
                let option = detail
                    .options
                    .iter()
                    .find(|o| o.item == ProductionItem::Unit { unit: wanted })?;
                if !matches!(
                    option.buy_block,
                    None | Some(legatum_core::economy::BuyBlock::NotEnoughGold)
                ) {
                    return None;
                }
                option
                    .buy_cost
                    .filter(|cost| *cost <= available)
                    .map(|cost| (wanted, cost))
            });
            if let Some((wanted, cost)) = replacement {
                if wanted != *unit {
                    crate::buy_trace::replace_item(
                        self.view().me,
                        *city,
                        ProductionItem::Unit { unit: *unit },
                        ProductionItem::Unit { unit: wanted },
                    );
                }
                if rescue.is_some() {
                    crate::buy_trace::mark_floor_rescue(
                        self.view().me,
                        *city,
                        ProductionItem::Unit { unit: wanted },
                    );
                }
                *unit = wanted;
                gold = available - cost;
            } else {
                removed.insert(index);
                gold = available;
            }
        }
        self.orders = orders
            .into_iter()
            .enumerate()
            .filter_map(|(i, o)| (!removed.contains(&i)).then_some(o))
            .collect();
        self.gold_left = gold;
    }

    /// Value of a fresh military purchase per Gold. The legacy branch is deliberately exact:
    /// turning the gene off restores the former strength-per-Gold comparisons.
    pub(crate) fn military_purchase_score(&self, kind: UnitKind, cost: i32, pos: Hex) -> i32 {
        if self.decisions.purchase_generation_floor > 0
            && self.difficulty == AiDifficulty::Hard
            && composition::is_combat(kind)
            && let Some(city) = self.ctx.own_cities.iter().copied().find(|c| c.pos == pos)
        {
            if let Some((rescue, _)) = self.purchase_rescue(city, self.gold_left.max(cost)) {
                return if kind == rescue {
                    1_000_000 - cost
                } else {
                    -1_000_000
                };
            }
            if self.purchase_generation(city, kind) != Some(kind) {
                return -1_000_000;
            }
        }
        if self.decisions.hard_purchase_scorer == 0 || self.difficulty != AiDifficulty::Hard {
            return self.purchase_combat_strength(kind, pos).max(0) * 1000 / cost.max(1);
        }
        if data::is_aircraft(kind) {
            return self.purchase_combat_strength(kind, pos).max(0) * 1000 / cost.max(1);
        }
        let v = self.view();
        let stats = v.rule_experiments.unit_stats_for(self.faction, kind);
        let strength = stats.strength.max(stats.ranged);
        let off = v.rule_experiments.faction_rules_off;
        let foes: Vec<_> = v
            .units
            .iter()
            .filter(|e| {
                e.military
                    && composition::is_combat(e.kind)
                    && !v.same_team(v.me, e.owner)
                    && !(e.owner.is_barbarian() && v.barbarians_spare_me())
                    && e.pos.distance(pos) <= 12
            })
            .collect();
        // A newly trained old unit still needs the whole researched refit chain. Its
        // purchase price alone understates what it costs to field in this war.
        let mut refit = 0;
        let mut next = kind;
        while let Some(to) = data::unit_stats(next).upgrades_to {
            if !data::unit_stats(to)
                .requires
                .is_none_or(|t| v.research.known.contains(&t))
            {
                break;
            }
            refit += legatum_core::rules::units::upgrade_cost_for(self.faction, next).unwrap_or(0);
            next = to;
        }
        let price = cost.saturating_add(refit).max(1);
        let mobility = 100 + (stats.moves - 2).max(0) * 18;
        if foes.is_empty() {
            return ((i64::from(strength) * i64::from(strength) * i64::from(mobility) * 10)
                / i64::from(price))
            .min(i64::from(i32::MAX)) as i32;
        }
        // Reuse the engine's exact roll forecast for a visible enemy this unit can
        // attack from its purchase tile. More distant and peaceful rivals are valued
        // with the same combat strength and damage functions until a legal fight exists.
        let projected = foes
            .iter()
            .any(|e| v.at_war(v.me, e.owner) && e.pos.distance(pos) <= 3)
            .then(|| {
                let mut projected = v.clone();
                let recruit = purchased_defender(
                    v,
                    self.ctx
                        .own_cities
                        .iter()
                        .copied()
                        .find(|c| c.pos == pos)
                        .expect("purchase city"),
                    kind,
                );
                let mut units = v.units.to_vec();
                units.push(recruit);
                projected.units = units.into();
                projected
            });
        let mut value = 0i64;
        for enemy in &foes {
            let own_bonus =
                legatum_core::combat::class_bonus_for_rules(off, self.faction, kind, enemy.kind).1;
            let enemy_bonus = legatum_core::combat::class_bonus_for_rules(
                off,
                v.faction_of(enemy.owner),
                enemy.kind,
                kind,
            )
            .1;
            let attack = if stats.ranged > 0 {
                stats.ranged
            } else {
                stats.strength
            };
            let mode = if stats.ranged > 0 && enemy.pos.distance(pos) <= stats.range {
                legatum_core::AttackMode::Ranged
            } else {
                legatum_core::AttackMode::Melee
            };
            let dealt = projected
                .as_ref()
                .filter(|_| enemy.pos.distance(pos) <= 3)
                .and_then(|projected| {
                    legatum_core::forecast_attack_mode(
                        projected,
                        legatum_core::UnitId(u32::MAX),
                        enemy.pos,
                        mode,
                    )
                })
                .map(|f| f.expected_damage_to_defender)
                .unwrap_or_else(|| {
                    legatum_core::combat::base_damage(
                        attack + own_bonus,
                        enemy.strength + enemy_bonus,
                    )
                });
            let received = legatum_core::combat::base_damage(
                enemy.strength.max(enemy.ranged) + enemy_bonus,
                stats.strength + own_bonus,
            );
            // A shooter can fire before the reply, but a unit that dies in its first
            // exchange has little value in a second fight. A city rescue is checked by
            // defense_risk at its call site and is never rejected by this estimate.
            let endurance = if received >= data::UNIT_HP {
                15
            } else if received * 2 >= data::UNIT_HP {
                55
            } else {
                100
            };
            value += i64::from(dealt.max(1)) * i64::from(endurance);
        }
        ((value * i64::from(mobility) / i64::try_from(foes.len()).unwrap_or(1) / i64::from(price))
            .min(i64::from(i32::MAX))) as i32
    }

    /// Keep Gold briefly when the only affordable troop is much worse per Gold
    /// than a trainable one. A purchase in a falling city must extend its survival
    /// clock; Gold and upgrade comparisons still decide whether the delay is worth it.
    pub(crate) fn military_purchase_viable(
        &self,
        city: &CityView,
        kind: UnitKind,
        cost: i32,
    ) -> bool {
        if self.decisions.hard_purchase_scorer == 0 || self.difficulty != AiDifficulty::Hard {
            return true;
        }
        let before = self.defense_risk(city, None);
        if before.falls {
            let after = self.defense_risk(city, Some(purchased_defender(self.view(), city, kind)));
            if self.decisions.siege_survival_clock == 0 {
                return !after.falls;
            }
            if !after.holds_longer_than(before) {
                return false;
            }
        }
        // A strong city shields its garrison through the first exchanges. When a
        // front arrives next turn, an affordable old body can buy time there
        // even if a stronger recruit is several income turns away.
        let strength = self
            .view()
            .rule_experiments
            .unit_stats_for(self.faction, kind)
            .strength;
        if strength > 0
            && city.hp >= city.max_hp / 2
            && city.strength >= strength.saturating_mul(4)
            && self
                .door_fronts()
                .iter()
                .any(|front| front.city == city.id && front.arrival <= 1 && front.gap() > 0)
        {
            return true;
        }
        let Some(detail) = city.detail.as_ref() else {
            return true;
        };
        let income = self.view().economy.gold_per_turn.total.max(0);
        let horizon_gold = self.gold_left.saturating_add(income.saturating_mul(5));
        let score = self.military_purchase_score(kind, cost, city.pos);
        !detail.options.iter().any(|option| {
            let ProductionItem::Unit { unit } = option.item else {
                return false;
            };
            composition::is_combat(unit)
                && matches!(
                    option.buy_block,
                    None | Some(legatum_core::economy::BuyBlock::NotEnoughGold)
                )
                && option.buy_cost.is_some_and(|quote| {
                    quote > cost
                        && quote <= horizon_gold
                        && self.military_purchase_score(unit, quote, city.pos)
                            >= score.saturating_mul(2)
                })
        })
    }

    /// The researched upgrade of a unit already in the army is due shortly. Spend now only
    /// when the city is forecast to fall before that upgrade can arrive.
    pub(crate) fn military_upgrade_hold(&self, city: &CityView) -> bool {
        if self.decisions.hard_purchase_scorer == 0 || self.difficulty != AiDifficulty::Hard {
            return false;
        }
        let v = self.view();
        let Some(tech) = v.research.current else {
            return false;
        };
        let Some(research_turns) = v.research.turns_left.filter(|n| *n <= 3) else {
            return false;
        };
        if self
            .defense_risk(city, None)
            .turns_to_fall
            .is_some_and(|fall| fall <= research_turns)
        {
            return false;
        }
        if self.decisions.named_fund > 0
            && self.named_fund().is_some_and(|fund| {
                matches!(fund.kind, crate::gold_hold::FundKind::Upgrade(t) if t == tech)
                    && !fund.ready
            })
        {
            return true;
        }
        self.ctx.own_units.iter().any(|u| {
            composition::is_combat(u.kind)
                && u.contract_until.is_none()
                && data::unit_stats(u.kind)
                    .upgrades_to
                    .is_some_and(|to| data::unit_stats(to).requires == Some(tech))
        })
    }

    /// A human city's one delegated choice, using Hard's production valuation over its
    /// fogged view. Focus multiplies a category's value; it never changes the rules.
    fn automated_choice(
        &self,
        city: CityId,
        focus: CityBuildFocus,
        planned: &[ProductionItem],
    ) -> Option<ProductionItem> {
        let (ci, c) = self
            .ctx
            .own_cities
            .iter()
            .enumerate()
            .find(|(_, c)| c.id == city)?;
        let d = c.detail.as_ref()?;
        let mut tally = Tally {
            army: 0,
            rangers: 0,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: self.count_good_sites(4),
        };
        for u in &self.ctx.own_units {
            tally.add(ProductionItem::Unit { unit: u.kind });
        }
        for own in &self.ctx.own_cities {
            if let Some(item) = own.detail.as_ref().and_then(|detail| detail.producing) {
                // The build completing this turn does not remain in the pipeline.
                if own.id != city {
                    tally.add(item);
                }
            }
        }
        for item in planned {
            tally.add(*item);
        }
        if focus == CityBuildFocus::Balanced {
            let picked = self.choose_item(ci, c, &tally);
            if !picked.is_some_and(|item| {
                (d.producing == Some(item) && !matches!(item, ProductionItem::Unit { .. }))
                    || d.options
                        .iter()
                        .find(|o| o.item == item)
                        .is_some_and(|o| o.resource_blocker.is_some())
            }) {
                return picked;
            }
        }
        // A focus builds its own kind first (user, 2026-11-01: a Military city built a Farm and a Granary): any
        // available favored item outranks every other item, and the rest only fill in when none is available.
        let scored: Vec<(bool, i32, ProductionItem)> = d.options.iter().filter_map(|opt| {
            if matches!(opt.item, ProductionItem::Project { .. })
                // Expansion remains an explicit part of Balanced, not a distraction
                // from a specialized focus.
                || (focus != CityBuildFocus::Balanced
                    && matches!(opt.item, ProductionItem::Unit { unit: UnitKind::Settler }))
                || (matches!(focus, CityBuildFocus::Science | CityBuildFocus::Gold) && is_military_unit(opt.item))
                || opt.resource_blocker.is_some()
                || !self.view().rule_experiments.available_to_player(self.faction, self.view().era_of(self.view().me), opt.item)
                || matches!(opt.item, ProductionItem::Unit { unit } if self.troop_unfunded(ci, c, unit))
                || (d.producing == Some(opt.item) && !matches!(opt.item, ProductionItem::Unit { .. }))
            { return None; }
            let favored = match (focus, opt.item) {
                (CityBuildFocus::Science, ProductionItem::Building { building }) => self.view().rule_experiments.building_stats_for(self.faction, building).yields.science > 0,
                (CityBuildFocus::Gold, ProductionItem::Building { building }) => self.view().rule_experiments.building_stats_for(self.faction, building).yields.gold > 0,
                (CityBuildFocus::Military, item) => is_military_unit(item),
                _ => false,
            };
            let value = self.item_value(ci, c, opt, &tally) - self.wonder_tile_penalty(opt);
            // Hard may see no use for one more unit in peacetime; the player's focus says otherwise.
            let value = if favored { value.max(1) } else { value };
            if value <= 0 { return None; }
            let score = value * 60 / (opt.turns.clamp(0, 400) + 60)
                + value * opt.production_banked.clamp(0, opt.cost.max(1)) / opt.cost.max(1) / 2;
            Some((favored, score, opt.item))
        }).collect();
        let any_favored = scored.iter().any(|(favored, _, _)| *favored);
        scored
            .iter()
            .filter(|(favored, _, _)| *favored || !any_favored)
            .max_by_key(|(_, score, item)| (*score, *item))
            .map(|(_, _, item)| *item)
    }

    pub(crate) fn production_defense_trace(&self) -> Vec<ProductionDefenseTrace> {
        self.ctx
            .own_cities
            .iter()
            .enumerate()
            .map(|(i, c)| ProductionDefenseTrace {
                city: c.id,
                threat: self.ctx.threat[i],
                local_defense: self.local_defense(c),
                falls: self.defense_risk(c, None).falls,
                exposure: self.threat_exposure(c),
                early_warning: self.early_warning_power(c),
            })
            .collect()
    }

    fn swiss_market_priority(&self) -> i32 {
        if self.difficulty == legatum_core::AiDifficulty::Hard
            && self.faction == Faction::Switzerland
            && self.goal_pending()
            && self.rule_on(legatum_core::experiments::FactionRule::SwissTolls)
        {
            self.decisions.swiss_market_priority
        } else {
            0
        }
    }

    /// A lone surviving city cannot spend its next build on a Settler while a
    /// wartime army was recently seen close to the gate. This also covers the
    /// turn after another city falls, when terrain sight hides its new owner.
    fn last_city_front(&self, city: &CityView) -> bool {
        self.ctx.own_cities.len() == 1
            && (self.ctx.enemy_units.iter().any(|enemy| {
                enemy.military
                    && !enemy.owner.is_barbarian()
                    && self.view().at_war(self.view().me, enemy.owner)
                    && enemy.pos.distance(city.pos) <= self.decisions.lone_city_radius
            }) || self.ctx.remembered_threats.iter().any(|enemy| {
                self.view().turn.saturating_sub(enemy.seen_turn) <= 5
                    && !enemy.owner.is_barbarian()
                    && self.view().at_war(self.view().me, enemy.owner)
                    && enemy.pos.distance(city.pos) <= self.decisions.lone_city_radius
            }))
    }

    /// What losing this unit would cost us, in Production: its Production value scaled by
    /// health, plus the stake it holds. The only defender standing in a city at risk
    /// carries that city (a newly captured city in resistance, a damaged one, or one with
    /// enemies in sight or recent memory), and a unit beside an enemy city we are at war
    /// with carries part of the siege.
    pub(crate) fn unit_keep_value(&self, u: &UnitView) -> i32 {
        let mut value = crate::tuned(
            self.view()
                .rule_experiments
                .unit_stats_for(self.faction, u.kind)
                .cost,
        ) * u.hp.max(1)
            / data::UNIT_HP;
        if let Some((i, city)) = self
            .ctx
            .own_cities
            .iter()
            .enumerate()
            .find(|(_, c)| c.pos == u.pos)
        {
            let resistance = city.detail.as_ref().is_some_and(|d| d.resistance > 0);
            let enemies_near = self
                .ctx
                .threat_units
                .iter()
                .any(|e| e.military && e.pos.distance(city.pos) <= 6)
                || self
                    .ctx
                    .remembered_threats
                    .iter()
                    .any(|o| o.pos.distance(city.pos) <= 6)
                || self
                    .ctx
                    .enemy_cities
                    .iter()
                    .any(|e| e.pos.distance(city.pos) <= 6);
            let at_risk =
                self.ctx.threat[i] > 0 || city.hp < city.max_hp || resistance || enemies_near;
            let stake = city_keep_value(city);
            value += if at_risk { stake } else { stake / 4 };
        }
        if self
            .ctx
            .enemy_cities
            .iter()
            .any(|c| c.pos.distance(u.pos) <= 2 && c.hp > 0)
        {
            value += value / 2;
        }
        value
    }

    /// Disband the units we value least until this turn's upkeep is payable. Leaving the
    /// shortfall to the engine is no bargain: a bankrupt treasury resets to zero and loses its
    /// costliest unit wherever it stands (`rules::cities::bankruptcy_victim`), and a
    /// structural deficit repeats that every turn (campaign funding audit, 2026-10-04: an
    /// AI that deferred to the engine lost 978 of its costliest units to bankruptcy in 100
    /// games, against 68 on master). Units far from the front go first and committed
    /// campaign units last. Disbanded units leave `ctx.own_units` immediately so later
    /// upkeep and army counts this turn see the saving.
    fn settle_insolvency(&mut self, deficit: i32) {
        if deficit <= 0 {
            return;
        }
        // Least valuable first; at equal value the weaker fighter goes (a Mongols
        // Wayfinder costs a Spearman's 40 but can't hold anything).
        let mut candidates: Vec<(bool, bool, bool, i32, i32, legatum_core::UnitId)> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && !self.has_acted(u.id))
            .map(|u| {
                (
                    self.campaign_protects(u.id),
                    self.defending(u.pos),
                    self.at_front(u.pos),
                    self.unit_keep_value(u),
                    u.strength.max(u.ranged),
                    u.id,
                )
            })
            .collect();
        candidates.sort();
        let mut units = self.upkeep_unit_count() + self.planned_military_purchases();
        let (mut saved, mut chosen) = (0, Vec::new());
        for (_, _, _, _, _, id) in candidates {
            if saved >= deficit {
                break;
            }
            let saving =
                self.engine_military_upkeep(units) - self.engine_military_upkeep(units - 1);
            if saving <= 0 {
                break;
            }
            units -= 1;
            saved += saving;
            chosen.push(id);
        }
        if saved < deficit {
            // The engine disbands its victim anyway; extra disbands would only add losses.
            return;
        }
        for id in chosen {
            if self.unit_order(id, Order::DisbandUnit { unit: id }) {
                self.ctx.own_units.retain(|u| u.id != id);
            }
        }
    }

    /// The engine's own forecast already says this turn ends with the treasury negative
    /// unless something changes (`view.economy.bankruptcy`, current queues/positions held
    /// fixed, no new orders). Unit orders resolve before the economy phase
    /// (`unit_phases_before_cities` precedes `cities::process_cities`), so disbanding now
    /// keeps this turn's upkeep payable and preempts the engine's bankruptcy disband when
    /// that loses less. (rush-defense, user run 2026-10-04: the old rule disbanded three
    /// units in one turn to cover 3 Gold, including the only garrison of the city captured
    /// the turn before.)
    fn disband_insolvent_unit(&mut self) {
        if let Some(b) = &self.view().economy.bankruptcy {
            self.settle_insolvency(-b.gold.total - self.campaign_income_delta());
        }
    }

    pub(crate) fn campaign_income_delta(&self) -> i32 {
        self.orders
            .iter()
            .filter_map(|order| {
                let Order::SetProduction { city, item } = order else {
                    return None;
                };
                let c = self.ctx.own_cities.iter().find(|c| c.id == *city)?;
                let d = c.detail.as_ref()?;
                let new_gold = d
                    .options
                    .iter()
                    .find(|o| o.item == *item)?
                    .yields
                    .total
                    .gold;
                Some(
                    self.remitted_city_gold(c, new_gold)
                        - self.remitted_city_gold(c, d.yields.total.gold),
                )
            })
            .sum()
    }

    /// The production option shows gross city Gold. India's provincial court sends none
    /// until its Revenue Office is complete, then only half, rounded like the engine.
    pub(crate) fn remitted_city_gold(&self, city: &CityView, gross: i32) -> i32 {
        if self.faction == Faction::India && !city.is_capital {
            if city
                .detail
                .as_ref()
                .is_some_and(|d| data::has_building(&d.buildings, BuildingKind::Market))
            {
                gross / 2
            } else {
                0
            }
        } else {
            gross
        }
    }

    /// A campaign's treasury clock owns the production choice before bankruptcy reacts.
    /// Changing the active build changes automatic tile focus this turn; Gold projects or
    /// Markets add income, while interrupting a finishing nonessential troop avoids upkeep.
    fn fund_campaign_clock(&mut self) {
        // A siege that captures on the last payable turn still needs to hold through
        // resistance, when the new city yields no income. Extend the runway early
        // with a unit outside the committed force, before bankruptcy chooses a captor.
        if let Some(active) = &self.campaigns.active
            && let Some(deadline) = active.deadline
            && let Some(city) = self.view().cities.iter().find(|c| c.id == active.target)
            && let Some((capture_turns, _)) = self.campaign_siege_estimate(city, &active.committed)
        {
            let holding_turns = (data::CAPTURE_RESISTANCE_TURNS) + 1;
            if deadline
                <= self
                    .view()
                    .turn
                    .saturating_add(capture_turns.max(0) as u32 + holding_turns as u32)
            {
                let target = city.pos;
                let army = self.upkeep_unit_count();
                let candidate = self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| {
                        u.military && !self.has_acted(u.id) && !self.campaign_protects(u.id)
                    })
                    .filter(|_| {
                        self.engine_military_upkeep(army) > self.engine_military_upkeep(army - 1)
                    })
                    .min_by_key(|u| {
                        (
                            self.defending(u.pos),
                            u.pos.distance(target) <= 4,
                            self.unit_keep_value(u),
                            u.id,
                        )
                    })
                    .map(|u| u.id);
                if let Some(id) = candidate
                    && self.unit_order(id, Order::DisbandUnit { unit: id })
                {
                    self.ctx.own_units.retain(|u| u.id != id);
                }
            }
        }
        let deadline = self
            .campaigns
            .active
            .as_ref()
            .and_then(|c| c.deadline)
            .or_else(|| {
                self.campaigns
                    .held_capture
                    .as_ref()
                    .and_then(|_| self.campaign_deadline())
            });
        let Some(deadline) = deadline else { return };
        for c in &self.ctx.own_cities {
            let Some(d) = &c.detail else { continue };
            if d.resistance > 0 {
                continue;
            }
            let finishing_troop = d.producing.is_some_and(is_military_unit)
                && d.turns_to_complete.is_some_and(|n| {
                    n >= 0 && n as u32 <= deadline.saturating_sub(self.view().turn)
                });
            let best = d
                .options
                .iter()
                .filter(|o| !matches!(o.item, ProductionItem::Unit { .. }))
                .filter(|o| {
                    self.remitted_city_gold(c, o.yields.total.gold)
                        > self.remitted_city_gold(c, d.yields.total.gold)
                        || finishing_troop
                        || d.producing.is_none()
                })
                .max_by_key(|o| {
                    (
                        self.remitted_city_gold(c, o.yields.total.gold)
                            - self.remitted_city_gold(c, d.yields.total.gold),
                        -o.turns,
                        std::cmp::Reverse(o.item),
                    )
                });
            if let Some(opt) = best
                && d.producing != Some(opt.item)
            {
                self.orders.push(Order::SetProduction {
                    city: c.id,
                    item: opt.item,
                });
            }
        }
    }

    /// The forecast above runs before this turn's own spending: an emergency purchase that
    /// saves a falling city can itself be what tips net gold negative this turn (a bought
    /// unit's upkeep is paid the same turn it completes). Re-check after every purchase
    /// this turn is decided, using existing units (never the one just bought).
    fn disband_if_this_turn_overspent(&mut self) {
        // `gold_per_turn.total` already deducted upkeep for the units that existed at the
        // start of this turn (`view.units`, unaffected by ctx mutations below); credit that
        // back and charge the current count (after any disbands so far) plus this turn's
        // purchases, so each disband is actually reflected instead of only comparing
        // purchases against a fixed, stale income figure.
        let original = self
            .view()
            .units
            .iter()
            .filter(|u| u.owner == self.ctx.me && legatum_core::data::pays_upkeep(u.kind))
            .count() as i32;
        let original_upkeep = self.military_upkeep_at(original);
        let current = self.upkeep_unit_count();
        let projected_upkeep = self.military_upkeep_at(current + self.planned_military_purchases());
        let projected = self.gold_left + self.view().economy.gold_per_turn.total + original_upkeep
            - projected_upkeep
            + self.campaign_income_delta();
        self.settle_insolvency(-projected);
    }

    /// Budget upkeep at its actual next-unit boundary, including this turn's purchases.
    pub(crate) fn marginal_military_upkeep(&self) -> i32 {
        let units = self.upkeep_unit_count() + self.planned_military_purchases();
        self.military_upkeep_at(units + 1) - self.military_upkeep_at(units)
    }

    /// Own units that count toward unit upkeep: armed military, never Scouts or civilians.
    pub(crate) fn upkeep_unit_count(&self) -> i32 {
        self.ctx
            .own_units
            .iter()
            .filter(|u| legatum_core::data::pays_upkeep(u.kind) && u.contract_until.is_none())
            .count() as i32
    }

    pub(crate) fn planned_military_purchases(&self) -> i32 {
        self.orders
            .iter()
            .filter(|order| {
                matches!(order,
                    Order::BuyItem { item: ProductionItem::Unit { unit }, .. }
                    if legatum_core::data::pays_upkeep(*unit))
            })
            .count() as i32
    }

    fn military_upkeep_at(&self, units: i32) -> i32 {
        if self.faction != Faction::Switzerland {
            return units * legatum_core::data::military_upkeep(self.faction);
        }
        self.engine_military_upkeep(units)
    }

    /// The engine's upkeep for this many military units: the free allowance, the faction
    /// rate and Strategic Logistics (`economy::upkeep_breakdown`). What a disband saves.
    pub(crate) fn engine_military_upkeep(&self, units: i32) -> i32 {
        let base = legatum_core::data::paid_military_upkeep_rules(
            self.view().rule_experiments.faction_rules_off,
            self.faction,
            (units - legatum_core::data::FREE_MILITARY_UNITS).max(0),
        );
        if self
            .view()
            .research
            .known
            .contains(&legatum_core::TechId::MilitaryEngineering)
        {
            (3 * base + 3) / 4
        } else {
            base
        }
    }

    pub fn plan_cities(&mut self) {
        if self.decision_trace.is_some() {
            self.note_option(crate::DecisionOption::new(
                "plan",
                format!("named fund: {:?}", self.named_fund()),
                "audit",
            ));
        }
        self.tag_buys("siege_or_hold");
        self.fund_campaign_clock();
        self.disband_insolvent_unit();
        if self.opening.is_none() && self.primitive_gold_reserve().is_none()
            || self
                .opening
                .is_some_and(|o| o.policy.opening == crate::openings::Opening::GreedyExpand)
        {
            self.buy_emergency_defender();
        }
        self.tag_buys("emergency");
        if self.hard_defense() {
            // Rescue every threatened front before optional faction/title spending.
            // Each iteration consumes one city purchase slot, bounding the loop.
            while self.buy_emergency_defender() {}
            self.tag_buys("hard_rescue");
            self.buy_ready_defenders();
            self.tag_buys("ready_defenders");
        }
        // Gene `door_defense`: close each visible front's gap before optional refits.
        self.pass("door");
        self.plan_door_defense();
        self.tag_buys("door");
        self.pass("modernization");
        self.plan_modernization();
        self.tag_buys("modernization");
        self.pass("emergency_production");
        self.plan_emergency_production();
        self.tag_buys("emergency_production");
        let mut defensive_buys = self.purchase_orders();
        // An opening owns its initial investment sequence; once completed or abandoned,
        // retain the normal public-VP endgame planner rather than plateauing at its quota.
        if !self.rushing() && self.opening.is_none_or(|o| o.next_build.is_none()) {
            self.pass("faction_goal");
            self.plan_faction_goal();
            self.tag_buys("faction_goal");
            self.pass("victory");
            self.plan_victory();
            self.tag_buys("victory");
        }
        let primitive_buy = self.plan_primitive_buy();
        self.tag_buys("primitive");
        if !primitive_buy && self.opening.is_some() {
            self.plan_opening_buy();
            self.tag_buys("opening");
        }
        self.pass("rush_army");
        self.fund_rush_army();
        self.tag_buys("rush_army");
        // A purchased Settler consumes population before construction is validated.
        // Discard incompatible opening construction orders after choosing purchases.
        let settler_purchases: Vec<_> = self
            .orders
            .iter()
            .filter_map(|o| match o {
                Order::BuyItem {
                    city,
                    item:
                        ProductionItem::Unit {
                            unit: UnitKind::Settler,
                        },
                } if self.opening.is_some() => Some(*city),
                _ => None,
            })
            .collect();
        let ctx = &self.ctx;
        let mut tally = Tally {
            army: 0,
            rangers: 0,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: self.count_good_sites(4),
        };
        for u in &ctx.own_units {
            tally.add(ProductionItem::Unit { unit: u.kind });
        }
        for c in &ctx.own_cities {
            let d = c.detail.as_ref().expect("own city detail");
            let producing = self
                .orders
                .iter()
                .find_map(|o| match o {
                    Order::SetProduction { city, item } if *city == c.id => Some(*item),
                    _ => None,
                })
                .or(d.producing);
            // Gene `settler_slow_turns`: a Settler a slow city is still this many turns or more
            // from finishing cannot flood the map with idle Settlers soon, so it does not
            // hold a slot in the Settler gate's pipeline count (it counts once it is close).
            // Purchases still see it, so a Settler already under way is not bought twice.
            let slow = item_is_settler(producing)
                && self.decisions.settler_slow_turns > 0
                && d.options.iter().any(|o| {
                    item_is_settler(Some(o.item)) && o.turns >= self.decisions.settler_slow_turns
                });
            if let Some(item) = producing {
                tally.add(item);
            }
            tally.slow_settlers += i32::from(slow);
        }
        // Purchases are independent of the production queue. Count immediate units,
        // then choose one useful purchase before assigning the remaining construction.
        for order in &self.orders {
            if let Order::BuyItem { item, .. } = order {
                tally.add(*item);
            }
        }
        let before_threat = self.purchase_orders();
        self.pass("threat");
        self.plan_threat_defense();
        self.tag_buys("threat");
        defensive_buys.extend(
            self.purchase_orders()
                .into_iter()
                .filter(|o| !before_threat.contains(o)),
        );
        if !primitive_buy && self.opening.is_none() {
            self.plan_buy(&mut tally);
            self.tag_buys("routine");
        }

        self.pass("routine_production");
        self.start_settler_on_growth(&mut tally);
        let mut cities: Vec<(usize, &CityView)> =
            self.ctx.own_cities.iter().copied().enumerate().collect();
        // Scarce Oil goes to the highest-valued available use, regardless of which city
        // happens to have the lower id. The ordinary option scorer compares aircraft, ground
        // units and Research Labs in the same units; within_budget spends the city-order share
        // of supply in this order (upgrades have already claimed their supply).
        let oil = legatum_core::resources::Resource::Oil;
        let spare_oil = self
            .view()
            .supply
            .iter()
            .find(|s| s.resource == oil)
            .map_or(0, |s| s.spare.max(0));
        let can_assign = |c: &CityView| {
            c.detail.as_ref().is_some_and(|d| {
                (d.producing.is_none()
                    || matches!(d.producing, Some(ProductionItem::Project { .. }))
                    || self.levy_blocks(d.producing))
                    && d.resistance <= 0
                    && !self
                        .orders
                        .iter()
                        .any(|o| matches!(o, Order::SetProduction { city, .. } if *city == c.id))
            })
        };
        let oil_cities = cities
            .iter()
            .filter(|(_, c)| {
                can_assign(c)
                    && c.detail
                        .as_ref()
                        .is_some_and(|d| d.options.iter().any(|o| o.supply_need == Some(oil)))
            })
            .count() as i32;
        if spare_oil > 0 && oil_cities > spare_oil {
            cities.sort_by_key(|(ci, c)| {
                let bid = c.detail.as_ref().filter(|_| can_assign(c)).map_or(0, |d| {
                    d.options
                        .iter()
                        .filter(|o| o.supply_need == Some(oil))
                        .map(|o| {
                            let value = self.item_value(*ci, c, o, &tally)
                                * self.strategy.map_or(100, |s| s.production_weight(o.item))
                                / 100;
                            value.max(0) * 60 / (o.turns.clamp(0, 400) + 60)
                        })
                        .max()
                        .unwrap_or(0)
                });
                (-bid, c.id)
            });
        }
        for (ci, c) in cities {
            let d = c.detail.as_ref().expect("own city detail");
            let abandon_wonder = d.producing.is_some_and(data::is_great_work_item)
                && !self.work_plan.as_ref().is_some_and(|plan| {
                    plan.host == c.id && matches!(d.producing,
                        Some(ProductionItem::Building { building } | ProductionItem::Wonder { building, .. })
                        if building == plan.building)
                });
            let reconsider = self.decisions.modernization_value > 0
                && matches!(d.producing, Some(ProductionItem::Unit { unit }) if composition::is_combat(unit))
                && !self.door_cities.contains(&c.id);
            let mut city_tally = tally;
            if reconsider && let Some(ProductionItem::Unit { unit }) = d.producing {
                city_tally.army -= 1;
                city_tally.rangers -= i32::from(composition::is_ranged_line(unit));
            }
            let egypt_stalled = self.faction == Faction::Egypt
                && self.difficulty == AiDifficulty::Hard
                && self.decisions.egypt_food_buildings > 0
                && self.view().rule_experiments.egypt_construction()
                && matches!(d.producing, Some(ProductionItem::Building { .. }))
                && d.yields.total.food <= 2 * c.population;
            if (reconsider || abandon_wonder || egypt_stalled || d.producing.is_none()
                || matches!(d.producing, Some(ProductionItem::Project { .. }))
                || self.levy_blocks(d.producing))
                && d.resistance <= 0
                && !self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::SetProduction { city, .. } if *city == c.id))
                && let Some(item) = self.choose_item(ci, c, &city_tally)
                && d.producing != Some(item)
                // A tile improvement bought this turn is done: it can't also be set as production.
                && !self.orders.iter().any(|o| {
                    matches!(o, Order::BuyItem { city, item: bought }
                        if *city == c.id && *bought == item && matches!(item, ProductionItem::Improvement { .. }))
                })
            {
                self.orders.push(Order::SetProduction { city: c.id, item });
                tally = city_tally;
                tally.add(item);
            }
            if egypt_stalled
                && !self.orders.iter().any(|o| {
                    matches!(o,
                    Order::SetProduction { city, .. } | Order::SetProductionQueue { city, .. }
                        if *city == c.id)
                })
            {
                // With nothing useful to train, park the building's own progress and
                // let this city grow until Food can fund it again.
                self.orders.push(Order::SetProductionQueue {
                    city: c.id,
                    queue: Vec::new(),
                });
            }
        }
        let bought_buildings: Vec<_> = self
            .orders
            .iter()
            .filter_map(|order| match order {
                Order::BuyItem {
                    city,
                    item: ProductionItem::Building { building },
                } => Some((*city, *building)),
                _ => None,
            })
            .collect();
        self.orders.retain(|order| match order {
            Order::SetProduction {
                city,
                item: ProductionItem::Building { building },
            } if bought_buildings.contains(&(*city, *building)) => false,
            Order::SetProduction {
                city,
                item: ProductionItem::Building { building },
            } if settler_purchases.contains(city) => self
                .ctx
                .own_cities
                .iter()
                .find(|c| c.id == *city)
                .is_some_and(|c| {
                    c.population > legatum_core::data::building_min_population(*building)
                }),
            _ => true,
        });
        self.drop_settler_set_after_settler_buy();
        // The reserve ledger closes the turn: troops the runway can't pay for pause in
        // non-front cities, optional purchases that would leave it short are withdrawn, and
        // only then are far-from-front units cut (`funding.rs`).
        self.pass("fund_production_runway");
        self.fund_production_runway();
        self.veto_unfunded_purchases(&defensive_buys);
        self.veto_plan_reserve(&defensive_buys);
        self.trim_unfunded_army();
        self.disband_if_this_turn_overspent();
    }

    /// A bought Settler is built at once and takes a population: setting the same city to build
    /// another Settler in the same turn is rejected when that leaves it under the Settler
    /// minimum (turn 10, Mongols, seed 31: `era_research.rs` caught the pair of orders).
    fn drop_settler_set_after_settler_buy(&mut self) {
        let bought: Vec<_> = self
            .orders
            .iter()
            .filter_map(|o| match o {
                Order::BuyItem {
                    city,
                    item:
                        ProductionItem::Unit {
                            unit: UnitKind::Settler,
                        },
                } => Some(*city),
                _ => None,
            })
            .collect();
        if bought.is_empty() {
            return;
        }
        let own_cities = &self.ctx.own_cities;
        self.orders.retain(|o| match o {
            Order::SetProduction {
                city,
                item:
                    ProductionItem::Unit {
                        unit: UnitKind::Settler,
                    },
            } if bought.contains(city) => own_cities
                .iter()
                .find(|c| c.id == *city)
                .is_some_and(|c| c.population > data::SETTLER_MIN_POP),
            _ => true,
        });
    }

    /// Diagnostics (`buy_trace`): credit this turn's untagged purchases to `pass`.
    pub(crate) fn tag_buys(&self, pass: &'static str) {
        crate::buy_trace::note(self.view().me, &self.orders, pass);
    }

    fn purchase_orders(&self) -> Vec<Order> {
        self.orders
            .iter()
            .filter(|o| matches!(o, Order::BuyItem { .. }))
            .cloned()
            .collect()
    }

    /// Fund upkeep before the next troop would bankrupt the army; the opening resumes its
    /// unfinished list. The interrupted troop keeps its progress banked (per-item banking).
    fn fund_rush_army(&mut self) {
        if !self.rushing()
            || self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military && u.kind != UnitKind::Scout)
                .count()
                < 2
        {
            return;
        }
        let item = ProductionItem::Building {
            building: BuildingKind::Market,
        };
        for city in self.ctx.own_cities.clone() {
            let Some(d) = city.detail.as_ref() else {
                continue;
            };
            if (self.faction == Faction::India && city.is_capital)
                || (self.faction != Faction::India && !city.is_capital)
                || d.producing == Some(item)
                || d.buildings.contains(&BuildingKind::Market)
                || !d
                    .producing
                    .or_else(|| self.opening?.next_build)
                    .is_some_and(is_military_unit)
            {
                continue;
            }
            let Some(market) = d.options.iter().find(|o| o.item == item) else {
                continue;
            };
            let income = self.view().economy.gold_per_turn.total;
            let upkeep = data::military_upkeep(self.faction);
            let build_turns = d.turns_to_complete.unwrap_or(0).max(0);
            let production = market.yields.total.production.max(1);
            let market_turns = (market.cost + production - 1) / production;
            let needed = (-income).max(0) * build_turns + (upkeep - income).max(0) * market_turns;
            if self.gold_left >= needed {
                continue;
            }
            self.orders.retain(|order| {
                !matches!(order,
                Order::SetProduction { city: id, .. } if *id == city.id)
            });
            self.orders.push(Order::SetProduction {
                city: city.id,
                item,
            });
        }
    }

    /// Greedy count of good, non-overlapping settling sites near own cities, up to `cap`.
    pub fn count_good_sites(&self, cap: i32) -> i32 {
        let ctx = &self.ctx;
        let mut sites: Vec<(i32, Hex)> = ctx
            .site_score
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                let s = (*s)?;
                let h = ctx.hex_at(i);
                let near = ctx.own_cities.iter().any(|c| {
                    c.pos.distance(h) <= self.decisions.settler_max_site_distance
                        && ctx.reachable(c.pos, h)
                });
                (s >= self.decisions.settler_min_site_score && near && !ctx.in_danger(h))
                    .then_some((s, h))
            })
            .collect();
        sites.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut picked: Vec<Hex> = Vec::new();
        for (_, h) in sites {
            if picked.len() as i32 >= cap {
                break;
            }
            if picked.iter().all(|p| {
                p.distance(h)
                    > ctx
                        .view
                        .rule_experiments
                        .min_city_gap(ctx.view.faction_of(ctx.me))
            }) {
                picked.push(h);
            }
        }
        picked.len() as i32
    }

    /// Armed units this seat wants, within its thin-levy cap (`levy.rs`).
    pub(crate) fn desired_army(&self) -> i32 {
        let wanted = self.uncapped_army();
        let wanted = if self.faction == Faction::India && self.decisions.india_network_guard > 0 {
            let linked = self
                .ctx
                .own_cities
                .iter()
                .filter(|city| {
                    self.ctx
                        .own_cities
                        .iter()
                        .any(|other| other.id != city.id && other.pos.distance(city.pos) <= 5)
                })
                .count() as i32;
            wanted.max(self.ctx.own_cities.len() as i32 + (linked + 1) / 2)
        } else {
            wanted
        };
        // Gene `vow_hold`: two defenders per city while our closed country's vow decides the race.
        let wanted = if self.vow_race_closed() {
            wanted.max(2 * self.ctx.own_cities.len() as i32)
        } else {
            wanted
        };
        let wanted = wanted + self.wary_guard();
        self.levy_now().map_or(wanted, |(_, cap)| wanted.min(cap))
    }

    /// Gene `swiss_playbook` 1+: a Hard Swiss city's Alpine Redoubt is its garrison (the
    /// designer walled every new city first and kept 2 to 5 troops across 4 to 22 cities).
    /// Gene `walls_garrison`: the same rule for every faction's Walls.
    pub(crate) fn redoubt_garrison(&self) -> bool {
        (self.decisions.walls_garrison > 0
            || self.decisions.swiss_playbook >= 1
                && self.faction == Faction::Switzerland
                && self.rule_on(legatum_core::experiments::FactionRule::SwissRedoubt))
            && self.difficulty == legatum_core::AiDifficulty::Hard
    }

    /// Walls' defensive work per Production relative to another home troop. The enemy units
    /// are only those in our view; a known nearby city supplies a conservative one-unit prior.
    /// A campaign gives the troop credit for attacking, so Walls cannot consume its field army.
    fn walls_garrison_ratio(&self, c: &CityView) -> i32 {
        if self.decisions.walls_garrison_value == 0 {
            return 0;
        }
        let rules = self.view().rule_experiments;
        let wall = ProductionItem::Building {
            building: BuildingKind::Walls,
        };
        let wall_cost = c
            .detail
            .as_ref()
            .and_then(|d| d.options.iter().find(|o| o.item == wall))
            .map_or_else(
                || rules.base_production_cost(self.faction, wall),
                |o| o.cost,
            )
            .max(1);
        let troop = c
            .detail
            .as_ref()
            .and_then(|d| {
                d.options
                    .iter()
                    .filter_map(|o| match o.item {
                        ProductionItem::Unit { unit }
                            if unit != UnitKind::Scout
                                && rules.unit_stats_for(self.faction, unit).military =>
                        {
                            Some((unit, o.cost))
                        }
                        _ => None,
                    })
                    .min_by_key(|(unit, cost)| {
                        if self.decisions.named_fund > 0 && self.difficulty == AiDifficulty::Hard {
                            let stats = rules.unit_stats_for(self.faction, *unit);
                            (
                                -i32::from(data::unit_era_tier(*unit)),
                                -stats.strength,
                                *cost,
                            )
                        } else {
                            (0, 0, *cost)
                        }
                    })
            })
            .unwrap_or((
                UnitKind::Spearman,
                rules.unit_stats_for(self.faction, UnitKind::Spearman).cost,
            ));
        let troop_strength = rules.unit_stats_for(self.faction, troop.0).strength;
        let (near_count, seen_strength, siege) = self
            .view()
            .units
            .iter()
            .filter(|u| {
                u.military
                    && !self.view().same_team(self.view().me, u.owner)
                    && !(u.owner.is_barbarian() && self.view().barbarians_spare_me())
                    && u.pos.distance(c.pos) <= 6
            })
            .fold((0_i32, 0_i32, false), |(count, strength, siege), u| {
                (
                    count + 1,
                    strength.max(u.strength.max(u.ranged)),
                    siege || u.siege || data::built_in_siege(u.kind),
                )
            });
        let known_front = self.view().cities.iter().any(|e| {
            !self.view().same_team(self.view().me, e.owner) && e.pos.distance(c.pos) <= 12
        });
        if near_count == 0 && !known_front && !c.walls {
            return 0;
        }
        let threat_strength = if near_count > 0 {
            seen_strength
        } else {
            troop_strength
        };
        let attacks = near_count.clamp(1, 3);
        let era = self.view().era_of(self.view().me);
        let open = legatum_core::combat::city_defense_for_rules(
            rules,
            self.faction,
            c.population,
            false,
            0,
            c.hp,
            era,
        )
        .total;
        let walled = legatum_core::combat::city_defense_for_rules(
            rules,
            self.faction,
            c.population,
            true,
            0,
            c.hp,
            era,
        )
        .total;
        let percent = legatum_core::combat::wall_damage_percent(1, u8::from(siege), false);
        let incoming_open = legatum_core::combat::base_damage(threat_strength, open);
        let incoming_walled =
            legatum_core::combat::base_damage(threat_strength, walled) * percent / 100;
        // Four attacks on city HP and two return shots are the short siege a new defender
        // can influence. Walls' 200 HP shield is represented by the saved damage per hit.
        let wall_defense = (incoming_open - incoming_walled).max(0) * 4
            + legatum_core::combat::base_damage(walled, threat_strength) * 2 * attacks.min(2);
        // The unit can take up to its own 100 HP, then fight back. Its other service is
        // damage it could deal to the current plan's target, from this city's front.
        let unit_defense =
            data::UNIT_HP + legatum_core::combat::base_damage(troop_strength, threat_strength);
        let plan = self.campaigns.plan.active.as_ref().map(|p| &p.kind);
        let plan_unit = match plan {
            Some(crate::plan::PlanKind::FieldStrike { target, .. })
            | Some(crate::plan::PlanKind::Snipe {
                settler: target, ..
            }) => self.view().units.iter().find(|u| u.id == *target),
            _ => None,
        };
        let plan_city = match plan {
            Some(crate::plan::PlanKind::Campaign { target, .. }) => {
                self.view().cities.iter().find(|e| e.id == *target)
            }
            Some(crate::plan::PlanKind::Contain { against }) => self
                .view()
                .cities
                .iter()
                .filter(|e| self.view().team_of(e.owner) == *against)
                .min_by_key(|e| e.pos.distance(c.pos)),
            _ => None,
        };
        let active_attack = plan_unit.is_some()
            || plan_city.is_some()
            || self.strategy == Some(crate::Strategy::Rush)
            || self.campaigns.plan.posture == Some(crate::plan::Posture::Military);
        let city_target = if plan_unit.is_some() {
            None
        } else {
            plan_city.or_else(|| {
                self.view()
                    .cities
                    .iter()
                    .filter(|e| !self.view().same_team(self.view().me, e.owner))
                    .min_by_key(|e| e.pos.distance(c.pos))
            })
        };
        let attack_damage = plan_unit
            .filter(|u| u.pos.distance(c.pos) <= 12)
            .map(|u| legatum_core::combat::base_damage(troop_strength, u.strength))
            .or_else(|| {
                city_target
                    .filter(|e| e.pos.distance(c.pos) <= 12)
                    .map(|e| {
                        let hit = legatum_core::combat::base_damage(troop_strength, e.strength);
                        hit * legatum_core::combat::wall_damage_percent(u8::from(e.walls), 0, false)
                            / 100
                    })
            })
            .unwrap_or(0);
        let offense = if active_attack {
            attack_damage * 4
        } else {
            attack_damage * self.knobs.aggression * 2 / 100
        };
        (wall_defense * troop.1.max(1) * self.decisions.walls_garrison_value
            / (wall_cost * (unit_defense + offense).max(1)))
        .clamp(0, 400)
    }

    /// The cities a troop must garrison: every city, or under `redoubt_garrison` only those
    /// with no Redoubt standing, under way or buildable (before Masonry, or a captured city
    /// that cannot build one yet).
    pub(crate) fn troop_garrison_cities(&self) -> i32 {
        if self.decisions.walls_garrison_value > 0 {
            let walls = ProductionItem::Building {
                building: BuildingKind::Walls,
            };
            return self
                .ctx
                .own_cities
                .iter()
                .filter(|c| {
                    self.walls_garrison_ratio(c) < 100
                        || !c.walls
                            && !c.detail.as_ref().is_some_and(|d| {
                                d.producing == Some(walls)
                                    || d.options.iter().any(|o| o.item == walls)
                            })
                })
                .count() as i32;
        }
        if !self.redoubt_garrison() {
            return self.ctx.own_cities.len() as i32;
        }
        let walls = ProductionItem::Building {
            building: BuildingKind::Walls,
        };
        self.ctx
            .own_cities
            .iter()
            .filter(|c| {
                !c.walls
                    && !c.detail.as_ref().is_some_and(|d| {
                        d.producing == Some(walls) || d.options.iter().any(|o| o.item == walls)
                    })
            })
            .count() as i32
    }

    pub(crate) fn uncapped_army(&self) -> i32 {
        let k = self.knobs;
        let cities = self.ctx.own_cities.len() as i32;
        if self.conquest_city().is_some() || self.contain_war().is_some() {
            // Home guards plus two capture units and three shooters; a numerical
            // garrison quota alone cannot pay for a city-strike siege, nor for a containment
            // march on the leader (playtest 8.4: rivals at war with it had no field army).
            // Gene `hollow_march`: a war on a hollow leader raises the field army a siege needs,
            // whatever the faction's usual war quota (the 8.9.17 Swiss wanted one unit more).
            // The lean war army was measured on Hard only (2026-11-06): Easy and Normal keep the
            // larger army they had.
            let base = if self.difficulty == legatum_core::AiDifficulty::Hard {
                self.decisions.army_war_extra
            } else {
                EASIER_WAR_EXTRA
            };
            let extra = if self.decisions.hollow_march & crate::hollow_march::QUOTA != 0
                && self.hollow_war_target().is_some()
            {
                base.max(crate::hollow_march::HOLLOW_WAR_EXTRA)
            } else {
                base
            };
            let extra = if self.mongol_horde() {
                extra.max(5)
            } else {
                extra
            };
            // Gene `aztecs_playbook` bit `AZTEC_LEAN_ARMY`: the designer's Aztecs fought every
            // war with a small current-era field army (4 to 9 units for up to 20 cities).
            let extra = if self.faction == Faction::Aztecs
                && self.decisions.aztecs_playbook & AZTEC_LEAN_ARMY != 0
            {
                extra.min(AZTEC_WAR_EXTRA)
            } else {
                extra
            };
            return cities + extra;
        }
        if let Some(o) = self.opening
            && (o.phase == crate::openings::Phase::Conquer
                || o.phase == crate::openings::Phase::Opening
                    && o.policy.opening == crate::openings::Opening::Rush)
        {
            return cities + o.policy.numbers.war_army;
        }
        if self.strategy == Some(crate::Strategy::Rush) {
            // One home defender per city plus a funded, mixed field force. The old
            // unbounded quota consumed the opening income before the attack arrived.
            return cities
                + self.decisions.army_rush_extra
                + i32::from(self.view().turn > self.decisions.rush_window_turn as u32);
        }
        // Hard funds garrisons and a faction-sized field contingent instead of continually
        // adding upkeep when no city is threatened. Visible threats restore the full quota;
        // plan_victory can separately fund a quoted sequence that wins a public Army title.
        // The opening reinforcement is handled before this routine production quota.
        if self.difficulty == legatum_core::AiDifficulty::Hard
            && !self.ctx.threat.iter().any(|t| *t > 0)
        {
            // Against a public VP leader, the garrison budget alone concedes the race.
            let pressure = if self.has_reachable_enemy_city() {
                self.conquest_pressure()
            } else {
                0
            };
            if self.decisions.army_banker > 0 {
                return self
                    .troop_garrison_cities()
                    .max(legatum_core::data::FREE_MILITARY_UNITS)
                    + i32::from(pressure >= 80);
            }
            // Gene `celts_playbook` part 1: a sheltered Celtic city needs no home guard.
            // Gene `walls_peace_quota`: a walled city needs none either. Gene
            // `walls_garrison_value`: the cities whose Walls beat a troop need none.
            let guarded = if self.decisions.walls_garrison_value > 0 {
                self.troop_garrison_cities()
                    .min(self.garrisoned_city_count())
            } else if self.decisions.walls_peace_quota > 0 && self.redoubt_garrison() {
                self.garrisoned_city_count().min(
                    self.troop_garrison_cities()
                        .max(legatum_core::data::FREE_MILITARY_UNITS),
                )
            } else {
                self.garrisoned_city_count()
            };
            return guarded
                + self.decisions.army_hard_extra
                + if k.military >= 70 {
                    self.decisions.army_military_bonus
                } else {
                    0
                }
                + (cities + 2) * pressure / 100;
        }
        // Aggressive empires keep a standing field army on top of their garrisons.
        let pressure = if self.has_reachable_enemy_city() {
            self.conquest_pressure()
        } else {
            0
        };
        // Gene `swiss_playbook`: only the cities a troop must garrison scale it.
        1 + self
            .troop_garrison_cities()
            .max(legatum_core::data::FREE_MILITARY_UNITS)
            .min(cities)
            * (20 + k.military + k.aggression / 2)
            / 70
            + k.aggression / 25
            + (cities + 2) * pressure / 100
    }

    /// Own military power standing within 2 of a city.
    pub(crate) fn local_defense(&self, c: &CityView) -> i32 {
        self.ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout && u.pos.distance(c.pos) <= 2)
            .map(|u| defense_power(self.view(), u).max(attack_power(self.view(), u)))
            .sum()
    }

    /// Burn a captured city only when the visible counterattack outweighs the troops that can
    /// hold it. Compare the loot and denied recapture against its expected retained value.
    pub(crate) fn plan_raze(&mut self) {
        let mut raze = Vec::new();
        for (i, city) in self.ctx.own_cities.iter().enumerate() {
            let Some(detail) = &city.detail else { continue };
            if !detail.captured || detail.raze_blocker.is_some() {
                continue;
            }
            if self.decisions.raze_value > 0 {
                if self.raze_by_value(i, city) {
                    raze.push(city.id);
                }
                continue;
            }
            let pressure = (self.ctx.threat[i] - self.ctx.barbarian_threat[i]).max(0);
            let defense = self.local_defense(city).max(1);
            if pressure <= defense
                || pressure.saturating_mul(100)
                    < defense.saturating_mul(self.decisions.raze_pressure_percent)
            {
                continue;
            }
            let loss_chance = ((pressure - defense) * 100 / pressure).clamp(0, 100);
            let keep = city_keep_value(city)
                + (crate::raze::held_vp_at_city(city) + self.own_capital_vp(city))
                    * crate::raze::VP_GOLD;
            let former_threatens = detail.former_owner.is_some_and(|owner| {
                self.ctx
                    .enemy_units
                    .iter()
                    .any(|u| u.owner == owner && u.military && u.pos.distance(city.pos) <= 3)
            });
            let denial = if former_threatens {
                keep * loss_chance / 100
            } else {
                0
            } + (self.rival_capital_vp(city) + self.capital_restore_vp(city))
                * crate::raze::VP_GOLD
                * loss_chance
                / 100;
            let loot = city.population.saturating_mul(10) / 2;
            let retained = keep * (100 - loss_chance) / 100;
            if loot + denial > retained {
                raze.push(city.id);
            }
        }
        for city in raze {
            self.orders.retain(|o| !matches!(o,
                Order::SetProduction { city: id, .. } | Order::SetProductionQueue { city: id, .. }
                | Order::BuyItem { city: id, .. } | Order::HireCompany { city: id, .. } if *id == city));
            self.orders.push(Order::Raze { city });
        }
    }

    /// The banked civilian is worth finishing while a visible force cannot take the city
    /// this turn. Both early threat preparation and emergency production use this price.
    pub(crate) fn keep_committed_settler(&self, c: &CityView) -> bool {
        let Some(d) = &c.detail else { return false };
        self.hard_defense()
            && self.decisions.keep_settler > 0
            && d.producing
                == Some(ProductionItem::Unit {
                    unit: UnitKind::Settler,
                })
            && d.options.iter().any(|o| {
                o.item
                    == ProductionItem::Unit {
                        unit: UnitKind::Settler,
                    }
                    && o.production_banked * 100 >= o.cost * self.decisions.keep_settler
            })
            && !self.defense_risk(c, None).falls
            && !self
                .ctx
                .threat_units
                .iter()
                .any(|e| e.military && e.kind != UnitKind::Scout && e.pos.distance(c.pos) <= 2)
    }

    /// A nearby rival or a broad unscouted land perimeter warrants an opening defender.
    /// Hard can use its starting garrison while settling a known safe site; fog alone need
    /// not postpone that investment. Ruins alone are not a threat.
    fn opening_exposed(&self, c: &CityView) -> bool {
        let near = |h: Hex| c.pos.distance(h) <= 6 && self.ctx.reachable(c.pos, h);
        self.ctx.enemy_cities.iter().any(|e| near(e.pos))
            || self.ctx.threat_units.iter().any(|e| near(e.pos))
            || ((self.difficulty != legatum_core::AiDifficulty::Hard
                || self.count_good_sites(1) == 0)
                && self
                    .ctx
                    .frontier
                    .iter()
                    .filter(|h| {
                        c.pos.distance(**h) <= 3
                            && self.ctx.reachable(c.pos, **h)
                            && h.neighbors()
                                .iter()
                                .any(|n| self.view().tile(*n).is_some_and(|t| !t.explored))
                    })
                    .take(11)
                    .count()
                    == 11)
    }

    /// Every colony is a whole city: a Production floor, Food, border growth, territory and a
    /// place to build, worth more than any single building. Under a turn limit it pays back
    /// only over the turns left after it is trained and founded, so its value fades (in
    /// percent) as the limit nears. Uncapped games have no deadline; the rising price and
    /// the remaining land decide when expansion stops.
    fn colony_payback_percent(&self, settler_turns: i32) -> i32 {
        let v = self.view();
        let Some(limit) = v.turn_limit else {
            return 100;
        };
        let payback = i64::from(100);
        let left =
            i64::from(limit) - i64::from(v.turn) - i64::from(settler_turns.max(0)) - i64::from(8);
        (100 * left.clamp(0, payback) / payback) as i32
    }

    /// Gene `colony_compound`: a colony priced like an improvement that yields a whole city.
    /// Its per-turn points (the improvement scale: 4 Food, 3 Production, 2 Gold) are the best
    /// site's potential as a young city works it over a 40-turn window (a little over a third
    /// of the radius-3 potential: ring one at founding, a claimed tile every few turns), less
    /// the citizen the Settler takes from home. A city also trains Settlers of its own, so the
    /// return compounds: one more city raises the empire's founding rate by a share of
    /// 1/cities. Returned on the Production it costs, as improvements are. Off while a
    /// conquest target is set (the army follows the intent) and at 0.
    fn colony_compound_value(
        &self,
        c: &CityView,
        opt: &ProductionOption,
        site: i32,
        cities: i32,
    ) -> i32 {
        let gene = self.decisions.colony_compound;
        if gene == 0
            || self.difficulty != legatum_core::AiDifficulty::Hard
            || self.conquest_city().is_some()
        {
            return 0;
        }
        let window = 40.min(self.turns_remaining() - opt.turns.max(0) - 8);
        if window <= 0 {
            return 0;
        }
        let citizen = c.detail.as_ref().map_or(8, |d| {
            legatum_core::economy::tile_score(d.yields.total, legatum_core::CityFocus::Balanced)
                / (c.population + 6).max(1)
        });
        let points = (site * 36 / 100 - citizen).max(0);
        let compound = 100 + 100 / cities.max(1);
        (i64::from(gene)
            * i64::from(50 + self.knobs.expansion)
            * i64::from(points)
            * i64::from(window)
            * i64::from(compound)
            / (2 * i64::from(opt.cost.max(1)) * 10_000)) as i32
    }

    /// The quoted cost includes the empire's trained-settler surcharge and the first-Settler
    /// discount. Reading it from our own options (or the greyed-out list of a population-1
    /// city) also works after a settler founds, dies or is captured, without AI memory.
    fn first_settler_pending(&self, c: &CityView) -> bool {
        let settler = ProductionItem::Unit {
            unit: UnitKind::Settler,
        };
        let v = self.view();
        let full = v
            .rule_experiments
            .base_production_cost(self.faction, settler);
        let first = full * data::FIRST_SETTLER_PERCENT / 100;
        c.detail.as_ref().is_some_and(|d| {
            d.options
                .iter()
                .map(|o| (o.item, o.cost))
                .chain(d.unavailable.iter().map(|u| (u.item, u.cost)))
                .any(|(item, cost)| item == settler && cost == first)
        })
    }

    fn rush_opening_item(&self, c: &CityView, t: &Tally) -> Option<ProductionItem> {
        if self.strategy != Some(crate::Strategy::Rush)
            || self.view().turn > self.decisions.rush_window_turn as u32
        {
            return None;
        }
        let d = c.detail.as_ref()?;
        let available = |item| d.options.iter().any(|o| o.item == item);
        let built = |building| {
            legatum_core::data::has_building(&d.buildings, building)
                || self.orders.iter().any(|o| {
                    matches!(o, Order::BuyItem {
                city, item: ProductionItem::Building { building: b }
            } if *city == c.id && legatum_core::data::building_satisfies(*b, building))
                })
        };
        let ladder: &[UnitKind] = if self.mongol_horde() {
            &composition::MOUNTED_LADDER
        } else {
            &composition::MELEE_LADDER
        };
        let melee = ladder
            .iter()
            .copied()
            .map(|unit| ProductionItem::Unit { unit })
            .find(|i| available(*i));
        if t.army < 2 {
            return melee;
        }
        // The production and income base pays for four field troops plus the garrison.
        // This ordering also applies to purchases, so they cannot drain the army budget
        // on a settler or an unrelated building while the opening force is incomplete.
        for building in [BuildingKind::Workshop, BuildingKind::Market] {
            let item = ProductionItem::Building { building };
            if !built(building) && available(item) {
                return Some(item);
            }
        }
        let upkeep = legatum_core::data::military_upkeep(self.faction);
        let funded =
            self.view().economy.gold_per_turn.total >= upkeep || self.gold_left >= 60 * upkeep;
        if funded && t.army < self.desired_army() {
            if !self.mongol_horde() && t.rangers < 2 {
                if let Some(item) = composition::RANGED_LADDER
                    .into_iter()
                    .map(|unit| ProductionItem::Unit { unit })
                    .find(|i| available(*i))
                {
                    return Some(item);
                }
                if t.army >= 3 {
                    return None;
                }
            }
            return melee;
        }
        None
    }

    /// A pre-Settler Scout or Granary (`pre_settler_build`) waits, banked, once its city
    /// reaches population 2 and the planner would choose a Settler there: the Settler starts at
    /// once, and the Scout or Granary resumes from its banked progress afterwards. Switching
    /// costs nothing under per-item banking.
    fn start_settler_on_growth(&mut self, tally: &mut Tally) {
        let settler = ProductionItem::Unit {
            unit: UnitKind::Settler,
        };
        let mut switches = Vec::new();
        for (ci, c) in self.ctx.own_cities.iter().enumerate() {
            let Some(d) = c.detail.as_ref() else {
                continue;
            };
            // A Spearman may be the city's defender; only Scout or food-building fillers wait.
            let filler = matches!(
                d.producing,
                Some(
                    ProductionItem::Unit {
                        unit: UnitKind::Scout
                    } | ProductionItem::Building {
                        building: BuildingKind::Granary | BuildingKind::RiceStorehouse
                    }
                )
            );
            if !filler
                || !d.turns_to_complete.is_some_and(|t| t > 1)
                || d.resistance > 0
                || !d.options.iter().any(|o| o.item == settler)
                || self.ctx.threat[ci] > self.local_defense(c)
                || !self.opening_admits_settler()
                || self.count_good_sites(1) == 0
                || self.choose_item(ci, c, tally) != Some(settler)
                || self.orders.iter().any(|o| {
                    matches!(o, Order::SetProduction { city, .. } | Order::SetProductionQueue { city, .. }
                        if *city == c.id)
                })
            {
                continue;
            }
            switches.push(c.id);
        }
        for city in switches {
            self.orders.push(Order::SetProduction {
                city,
                item: settler,
            });
            tally.add(settler);
        }
    }

    /// A Settler needs population 2 to be chosen (as in Civ 6). A population-1 city that
    /// wants one builds something short first (a Granary or Brood Hall, a Scout if it has none, or a
    /// Spearman when exposed), so the Settler starts soon after the city grows.
    fn pre_settler_build(&self, c: &CityView, tally: &Tally) -> Option<ProductionItem> {
        let d = c.detail.as_ref()?;
        let settler = ProductionItem::Unit {
            unit: UnitKind::Settler,
        };
        if c.population >= 2
            || !d.unavailable.iter().any(|u| u.item == settler)
            || !self.opening_admits_settler()
            || self.count_good_sites(1) == 0
        {
            return None;
        }
        let option = |item: ProductionItem| d.options.iter().find(|o| o.item == item);
        let soldier = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        if self.opening_exposed(c) && tally.army < 2 {
            return option(soldier).map(|o| o.item);
        }
        let scout = ProductionItem::Unit {
            unit: UnitKind::Scout,
        };
        let granary = ProductionItem::Building {
            building: BuildingKind::Granary,
        };
        let brood_hall = ProductionItem::Building {
            building: BuildingKind::RiceStorehouse,
        };
        // A Scout only when none is left (extra units cost upkeep: 3 Gold each for the
        // Switzerland); otherwise a food building (Food builds population, and a claimed tile
        // with it), which costs no upkeep.
        let cheapest = |candidates: &[(ProductionItem, bool)]| {
            candidates
                .iter()
                .filter(|(_, wanted)| *wanted)
                .filter_map(|(item, _)| option(*item))
                .filter(|o| o.turns > 0)
                .min_by_key(|o| o.turns)
                .map(|o| o.item)
        };
        // A food building first: it costs no upkeep and a Spearman is only the last resort.
        cheapest(&[
            (scout, tally.scouts == 0),
            (granary, true),
            (brood_hall, true),
        ])
        .or_else(|| cheapest(&[(soldier, true)]))
    }

    /// Egypt on the Nile builds with surplus Food, so a building city never grows. Choose Grow
    /// over an optional build when the next population pays for itself soon: the Food it costs
    /// (what is still missing from the threshold) against the extra surplus it brings, estimated
    /// as the city's doubled Food per worked tile less the 2 Food the citizen eats. Settlers and
    /// military units are never displaced (threat defense and expansion decide those), and an
    /// idle city grows.
    fn egypt_nile_choice(
        &self,
        c: &CityView,
        chosen: Option<ProductionItem>,
    ) -> Option<ProductionItem> {
        let rules = self.view().rule_experiments;
        if self.faction != Faction::Egypt || !rules.egypt_nile() {
            return chosen;
        }
        let Some(d) = c.detail.as_ref() else {
            return chosen;
        };
        let grow = ProductionItem::Project {
            project: legatum_core::IndustryProject::Grow,
        };
        if !d.options.iter().any(|o| o.item == grow) {
            return chosen;
        }
        if matches!(chosen, Some(ProductionItem::Unit { unit }) if unit == UnitKind::Settler
            || unit == UnitKind::HeirToTheThrone
            || composition::is_combat(unit))
        {
            return chosen;
        }
        if chosen.is_none() {
            return Some(grow);
        }
        let horizon = self.decisions.egypt_grow_payback;
        if horizon <= 0 {
            return chosen;
        }
        let view = self.view();
        let left = view.turn_limit.map_or(i64::from(horizon) * 3, |limit| {
            i64::from(limit.saturating_sub(view.turn))
        });
        let horizon = i64::from(horizon).min(left / 3);
        // Doubled Food in either mode: on a build it shows as Food eaten plus converted progress.
        let build_pct = i64::from(rules.egypt_nile_build_percent().max(1));
        let food = i64::from(d.yields.total.food)
            + i64::from(d.yields.total.production.max(0)) * 100 / build_pct;
        let tiles = d.yields.worked.len().max(1) as i64;
        let gain = (food / tiles - i64::from(legatum_core::data::FOOD_PER_POP)).max(1);
        let missing = i64::from((d.growth_threshold - d.food_stored).max(0));
        if missing <= gain * horizon {
            Some(grow)
        } else {
            chosen
        }
    }

    fn choose_item(&self, ci: usize, c: &CityView, tally: &Tally) -> Option<ProductionItem> {
        if let Some(trace) = &self.production_trace {
            trace.borrow_mut().push(ProductionTrace {
                city: c.id,
                chosen: None,
                options: Vec::new(),
                settler: None,
                gates: Vec::new(),
                early_gate: None,
            });
        }
        let door = if self.threat_aware()
            && self.decisions.army_door_production > 0
            && self.door_fronts().iter().any(|f| {
                f.city == c.id
                    && f.gap()
                        > self
                            .orders
                            .iter()
                            .filter_map(|order| match order {
                                Order::BuyItem {
                                    city,
                                    item: ProductionItem::Unit { unit },
                                } if *city == c.id => Some(self.modernization_power(
                                    *unit,
                                    0,
                                    data::UNIT_HP,
                                    false,
                                    c.pos,
                                )),
                                _ => None,
                            })
                            .sum::<i32>()
                    && self.view().units.iter().any(|u| {
                        u.military
                            && self.view().at_war(self.view().me, u.owner)
                            && u.pos.distance(c.pos) <= 6
                    })
            }) {
            c.detail.as_ref().and_then(|d| {
                d.options
                    .iter()
                    .filter(|o| {
                        o.turns > 0
                            && o.resource_blocker.is_none()
                            && matches!(o.item, ProductionItem::Unit { unit }
                                if crate::composition::is_melee_line(unit)
                                    || crate::composition::is_ranged_line(unit))
                    })
                    .max_by_key(|o| {
                        let ProductionItem::Unit { unit } = o.item else {
                            unreachable!()
                        };
                        (
                            data::unit_stats(unit)
                                .requires
                                .map(legatum_core::eras::era_of_tech)
                                .unwrap_or(0),
                            self.modernization_power(unit, 0, data::UNIT_HP, false, c.pos),
                            -o.turns,
                        )
                    })
                    .map(|o| o.item)
            })
        } else {
            None
        };
        let routine = door.or_else(|| self.choose_item_inner(ci, c, tally));
        let chosen = if door.is_some() {
            door
        } else {
            self.objective_production_choice(ci, c, tally, routine)
                .or(routine)
                .map(|item| self.forced_mix(ci, c, item))
        };
        let diwan = ProductionItem::Building {
            building: BuildingKind::ImperialDiwan,
        };
        let diwan_reserved = chosen == Some(diwan)
            && self.orders.iter().any(|order| {
                matches!(order,
                Order::SetProduction { item, .. } | Order::BuyItem { item, .. } if *item == diwan)
            });
        let unfunded_troop = matches!(chosen, Some(ProductionItem::Unit { unit })
            if self.faction == Faction::India && self.troop_unfunded(ci, c, unit));
        let chosen = if diwan_reserved || unfunded_troop {
            // Keep the city working without ordering a second Diwan or an unfunded troop.
            // A Revenue Office normally wins this fallback because it begins remittance.
            c.detail
                .as_ref()?
                .options
                .iter()
                .filter(|o| !matches!(o.item, ProductionItem::Unit { .. }) && o.item != diwan)
                .max_by_key(|o| (o.yields.total.gold, -o.turns, std::cmp::Reverse(o.item)))
                .map(|o| o.item)
        } else {
            chosen
        };
        // Opening and emergency overrides can bypass the scored-option loop. Apply the
        // same Food requirement to their final choice, using only this city's view.
        let chosen = if self.faction == Faction::Egypt
            && self.difficulty == AiDifficulty::Hard
            && self.decisions.egypt_food_buildings > 0
            && self.view().rule_experiments.egypt_construction()
            && chosen.is_some_and(|item| matches!(item, ProductionItem::Building { .. }))
            && c.detail
                .as_ref()
                .is_some_and(|d| d.yields.total.food <= 2 * c.population)
        {
            c.detail.as_ref().and_then(|d| {
                d.options
                    .iter()
                    .filter(|o| !matches!(o.item, ProductionItem::Building { .. }) && o.turns >= 0)
                    .max_by_key(|o| (self.item_value(ci, c, o, tally), -o.turns))
                    .map(|o| o.item)
            })
        } else {
            chosen
        };
        let chosen = self.egypt_nile_choice(c, chosen);
        if let Some(trace) = &self.production_trace
            && let Some(last) = trace.borrow_mut().last_mut()
        {
            last.chosen = chosen;
        }
        if self.decision_trace.is_some() {
            let scored = self
                .production_trace
                .as_ref()
                .and_then(|trace| trace.borrow().last().cloned());
            if let Some(scored) = scored {
                let mut ranking: Vec<i32> =
                    scored.options.iter().map(|(_, _, score)| *score).collect();
                ranking.sort_unstable_by(|a, b| b.cmp(a));
                let winner = ranking.first().copied();
                let runner_up = ranking.get(1).copied();
                if let Some(detail) = &c.detail {
                    for opt in &detail.options {
                        let mut row = crate::DecisionOption::new(
                            "production",
                            format!("city={:?} item={:?}", c.id, opt.item),
                            if chosen == Some(opt.item) {
                                "selected"
                            } else if scored.options.iter().any(|(item, _, _)| *item == opt.item) {
                                "considered_rejected"
                            } else {
                                "gated_out"
                            },
                        );
                        row.score = scored
                            .options
                            .iter()
                            .find(|(item, _, _)| *item == opt.item)
                            .map(|(_, _, score)| *score);
                        row.winner_score = winner;
                        row.runner_up_score = runner_up;
                        if row.outcome == "gated_out" {
                            row.gate = Some(
                                if let Some((_, gate)) =
                                    scored.gates.iter().find(|(item, _)| *item == opt.item)
                                {
                                    gate.clone()
                                } else if let Some(gate) = scored.early_gate {
                                    gate.to_owned()
                                } else if opt.item
                                    == (ProductionItem::Unit {
                                        unit: UnitKind::Settler,
                                    })
                                {
                                    scored.settler.map_or_else(
                                        || "prevaluation production override".to_owned(),
                                        |g| {
                                            [
                                                (!g.pop_ok, "population"),
                                                (!g.want, "settler demand"),
                                                (!g.time, "time"),
                                                (!g.room, "room"),
                                                (!g.developed, "development"),
                                                (!g.army_ok, "army"),
                                                (!g.sites_ok, "site score or distance"),
                                                (!g.pipeline_ok, "pipeline"),
                                            ]
                                            .into_iter()
                                            .find(|(blocked, _)| *blocked)
                                            .map_or(
                                                "prevaluation production override",
                                                |(_, reason)| reason,
                                            )
                                            .to_owned()
                                        },
                                    )
                                } else {
                                    "prevaluation production override or item-specific eligibility"
                                        .to_owned()
                                },
                            );
                        }
                        if row.outcome == "selected" {
                            row.selected_order = Some(Order::SetProduction {
                                city: c.id,
                                item: opt.item,
                            });
                        }
                        self.note_option(row);
                    }
                }
            }
        }
        chosen
    }

    /// Let a still-needed objective build bid against every routine choice, including an
    /// opening or army choice made before the general option loop. Both sides use the same
    /// production value and completion-time discount; a threatened city keeps its defender.
    fn objective_production_choice(
        &self,
        ci: usize,
        c: &CityView,
        tally: &Tally,
        routine: Option<ProductionItem>,
    ) -> Option<ProductionItem> {
        if self.difficulty != AiDifficulty::Hard
            || self.decisions.cheapest_objective == 0
            || self.ctx.threat[ci] > self.ctx.barbarian_threat[ci]
        {
            return None;
        }
        let detail = c.detail.as_ref()?;
        let score = |opt: &ProductionOption| {
            self.item_value(ci, c, opt, tally) * 60 / (opt.turns.clamp(0, 400) + 60)
        };
        let routine_score = routine
            .and_then(|item| detail.options.iter().find(|opt| opt.item == item))
            .map_or(0, &score);
        detail
            .options
            .iter()
            .filter(|opt| {
                opt.turns >= 0
                    && opt.resource_blocker.is_none()
                    && self.view().rule_experiments.available_to_player(
                        self.faction,
                        self.view().era_of(self.view().me),
                        opt.item,
                    )
                    && self.objective_build_value(c, opt.item) > 0
            })
            .max_by_key(|opt| (score(opt), std::cmp::Reverse(opt.item)))
            .filter(|opt| score(opt) > routine_score)
            .map(|opt| opt.item)
    }

    /// Experiment gene `force_mix`: a combat unit chosen in a city not under threat is swapped
    /// for the best buildable unit of the class furthest below the fixed target mix. Only the
    /// class changes, never whether a troop is built.
    fn forced_mix(&self, ci: usize, c: &CityView, item: ProductionItem) -> ProductionItem {
        use legatum_core::UnitClass::{Melee, Mounted, Ranged};
        let target: [i32; 3] = match self.decisions.force_mix {
            1 => [8, 1, 1],
            2 => [4, 2, 4],
            3 => [3, 5, 2],
            4 => [5, 5, 0],
            5 => [5, 2, 3],
            6 => self.sims_target_mix(),
            _ => return item,
        };
        let ProductionItem::Unit { unit } = item else {
            return item;
        };
        if !matches!(composition::class(unit), Some(Melee | Ranged | Mounted))
            || self.ctx.threat[ci] - self.ctx.barbarian_threat[ci] > 0
        {
            return item;
        }
        let slot = |kind: UnitKind| match composition::class(kind) {
            Some(Melee) => Some(0),
            Some(Ranged) => Some(1),
            Some(Mounted) => Some(2),
            _ => None,
        };
        let mut have = [0i32; 3];
        for u in &self.ctx.own_units {
            if let Some(i) = slot(u.kind) {
                have[i] += 1;
            }
        }
        for city in &self.ctx.own_cities {
            if city.id == c.id {
                continue;
            }
            let queued = self
                .orders
                .iter()
                .find_map(|o| match o {
                    Order::SetProduction { city: id, item } if *id == city.id => Some(*item),
                    _ => None,
                })
                .or_else(|| city.detail.as_ref().and_then(|d| d.producing));
            if let Some(ProductionItem::Unit { unit }) = queued
                && let Some(i) = slot(unit)
            {
                have[i] += 1;
            }
        }
        let total = have.iter().sum::<i32>() + 1;
        let sum: i32 = target.iter().sum();
        let ladders: [&[UnitKind]; 3] = [
            &composition::MELEE_LADDER,
            &composition::RANGED_LADDER,
            &composition::MOUNTED_LADDER,
        ];
        let buildable = |kind: UnitKind| {
            c.detail.as_ref().is_some_and(|d| {
                d.options
                    .iter()
                    .any(|o| o.item == ProductionItem::Unit { unit: kind })
            }) && !composition::obsolete(kind, |t| self.view().research.known.contains(&t))
        };
        let mut order = [0usize, 1, 2];
        // Largest deficit against the target share first.
        order.sort_by_key(|&i| (have[i] * sum - target[i] * total, i));
        for i in order {
            if target[i] == 0 {
                continue;
            }
            if let Some(kind) = ladders[i].iter().copied().find(|k| buildable(*k)) {
                return ProductionItem::Unit { unit: kind };
            }
        }
        item
    }

    /// The class mix the combat scenario simulator found best for our era against the army we
    /// face (`docs/sim-results/unit-classes/responses.txt`, Human policy, equal Production):
    /// each class seen in rival armies pulls toward the mix that beat that archetype, and the
    /// era's best worst-case mix anchors it, so an unseen enemy still gets a robust army.
    /// Melee/ranged/mounted, summing to about 10 per source.
    pub(crate) fn sims_target_mix(&self) -> [i32; 3] {
        // [robust, vs a melee line, vs a gunline (ranged screened by melee), vs horse]
        // The Machine Age and the Modern era (2026-11-14) have no scenario run yet: the Machine
        // Age reuses the Industrial row, the nearest era where the triangle still applies, and
        // the Modern era the World War row before it.
        const TABLE: [[[i32; 3]; 4]; legatum_core::eras::ERA_COUNT] = [
            [[8, 0, 0], [3, 3, 0], [2, 1, 3], [6, 1, 1]], // Ancient
            [[5, 1, 1], [3, 3, 0], [1, 1, 4], [6, 1, 1]], // Classical
            [[5, 1, 1], [2, 4, 0], [1, 1, 4], [6, 1, 1]], // Medieval
            [[5, 1, 1], [1, 5, 0], [0, 2, 4], [6, 1, 1]], // Renaissance
            [[3, 3, 1], [2, 5, 0], [1, 1, 4], [5, 2, 0]], // Industrial
            [[3, 3, 1], [2, 5, 0], [1, 1, 4], [5, 2, 0]], // Machine Age
            [[2, 4, 1], [0, 6, 1], [3, 1, 3], [4, 3, 0]], // World War
            [[2, 4, 1], [0, 6, 1], [3, 1, 3], [4, 3, 0]], // Modern
            [[1, 4, 2], [0, 5, 2], [3, 1, 3], [3, 4, 0]], // Future
        ];
        use legatum_core::UnitClass::{Melee, Mounted, Ranged};
        let v = self.view();
        let era = usize::from(v.research.era).min(TABLE.len() - 1);
        let mut seen = [0i32; 3];
        let mut note = |kind: UnitKind| match composition::class(kind) {
            Some(Melee) => seen[0] += 1,
            Some(Ranged) => seen[1] += 1,
            Some(Mounted) => seen[2] += 1,
            _ => {}
        };
        for e in &self.ctx.enemy_units {
            note(e.kind);
        }
        for o in &self.ctx.remembered_threats {
            note(o.kind);
        }
        for u in &v.units {
            if u.military
                && !u.owner.is_barbarian()
                && !v.same_team(v.me, u.owner)
                && self
                    .ctx
                    .own_cities
                    .iter()
                    .any(|c| c.pos.distance(u.pos) <= 10)
            {
                note(u.kind);
            }
        }
        let row = TABLE[era];
        // The robust mix weighs as much as two seen units, so a single sighting doesn't flip it.
        let robust_weight = 2;
        let screened = seen[1].min(seen[0]);
        let line = seen[0] - screened;
        // Screened shooters (a gunline) and loose ones both fall to the gunline answer.
        let gunline = screened * 2 + (seen[1] - screened);
        let horse = seen[2];
        std::array::from_fn(|k| {
            robust_weight * row[0][k] + line * row[1][k] + gunline * row[2][k] + horse * row[3][k]
        })
    }

    /// Gene `troop_runway`: training this troop here would leave fewer than the gene's turns of
    /// Gold runway once its upkeep is paid. A city defending itself is
    /// never held back.
    pub(crate) fn troop_unfunded(&self, ci: usize, c: &CityView, unit: UnitKind) -> bool {
        if self.faction == Faction::India
            && legatum_core::data::pays_upkeep(unit)
            && (self.difficulty != legatum_core::AiDifficulty::Hard
                || self.decisions.india_emergency_defense == 0
                || !self.funding_front(ci, c))
        {
            let net = self.planned_net_gold(1);
            if crate::funding::runway_of(self.gold_left, net).is_some_and(|r| r < 10) {
                return true;
            }
        }
        let turns = self.decisions.troop_runway;
        if turns <= 0
            || self.difficulty != legatum_core::AiDifficulty::Hard
            || !legatum_core::data::pays_upkeep(unit)
            || self.funding_front(ci, c)
        {
            return false;
        }
        let net = self.planned_net_gold(1);
        crate::funding::runway_of(self.gold_left, net).is_some_and(|r| r < turns.max(1) as u32)
    }

    /// Gene `aztecs_playbook` bit `AZTEC_WALLS`: an Aztec frontier city without Walls builds
    /// them first, before a rival at war finds it open. The frontier is public: an unallied
    /// rival empire's known city within `open_prize_radius` hexes over land we can reach. A
    /// Settler already under way keeps its slot.
    fn aztec_frontier_walls(&self, c: &CityView) -> Option<ProductionItem> {
        if !self.aztec_walls_playbook() || c.walls {
            return None;
        }
        let d = c.detail.as_ref()?;
        if item_is_settler(d.producing) {
            return None;
        }
        let walls = ProductionItem::Building {
            building: BuildingKind::Walls,
        };
        if !d.options.iter().any(|o| o.item == walls) {
            return None;
        }
        let v = self.view();
        let allied = |owner: legatum_core::PlayerId| {
            v.diplomacy.relations.iter().any(|r| {
                r.between(v.team, v.team_of(owner))
                    && r.state == legatum_core::diplomacy::RelationState::Alliance
            })
        };
        v.cities
            .iter()
            .any(|e| {
                !e.owner.is_barbarian()
                    && !v.same_team(v.me, e.owner)
                    && !allied(e.owner)
                    && e.pos.distance(c.pos) <= self.decisions.open_prize_radius
                    && self.ctx.reachable(c.pos, e.pos)
            })
            .then_some(walls)
    }

    /// Gene `aztecs_playbook` bit `AZTEC_WALLS` is on for this seat (Hard Aztecs).
    pub(crate) fn aztec_walls_playbook(&self) -> bool {
        self.faction == Faction::Aztecs
            && self.decisions.aztecs_playbook & AZTEC_WALLS != 0
            && self.difficulty == legatum_core::AiDifficulty::Hard
    }

    fn flower_wars_build_value(&self, city: &CityView, item: ProductionItem) -> i32 {
        if !self.rule_on(legatum_core::experiments::FactionRule::AztecsFlowerWars) {
            return 0;
        }
        let nearby = self.view().cities.iter().any(|c| {
            c.owner != self.view().me
                && !c.owner.is_barbarian()
                && self.view().team_of(c.owner) != self.view().team
                && c.pos.distance(city.pos) <= 12
        });
        let aztec_nearby = self.view().cities.iter().any(|c| {
            c.owner != self.view().me
                && self.view().team_of(c.owner) != self.view().team
                && self.view().faction_of(c.owner) == Faction::Aztecs
                && c.pos.distance(city.pos) <= 12
        });
        let earns = self.faction == Faction::Aztecs
            && nearby
            && matches!(item, ProductionItem::Unit { unit } if composition::is_melee_line(unit));
        let denies = aztec_nearby
            && (matches!(item, ProductionItem::Unit { unit } if composition::is_ranged_line(unit))
                || matches!(item, ProductionItem::Building { building } if data::building_family(building) == BuildingKind::Walls));
        if earns || denies {
            city.detail.as_ref().map_or(0, |d| d.growth_threshold)
                * self.decisions.flower_wars_value
                / 100
        } else {
            0
        }
    }

    fn record_option(&self, item: ProductionItem, value: i32, score: i32) {
        if let Some(trace) = &self.production_trace
            && let Some(last) = trace.borrow_mut().last_mut()
        {
            last.options.push((item, value, score));
        }
    }

    fn record_settler(&self, gates: SettlerGates) {
        if let Some(trace) = &self.production_trace
            && let Some(last) = trace.borrow_mut().last_mut()
        {
            last.settler = Some(gates);
        }
    }

    fn record_option_gate(&self, item: ProductionItem, gate: &'static str) {
        if let Some(trace) = &self.production_trace
            && let Some(last) = trace.borrow_mut().last_mut()
        {
            last.gates.push((item, gate.into()));
        }
    }

    fn early_choice(&self, item: ProductionItem, gate: &'static str) -> Option<ProductionItem> {
        if let Some(trace) = &self.production_trace
            && let Some(last) = trace.borrow_mut().last_mut()
        {
            last.early_gate = Some(gate);
        }
        Some(item)
    }

    fn choose_item_inner(&self, ci: usize, c: &CityView, tally: &Tally) -> Option<ProductionItem> {
        // Anticipatory defense precedes a greedy opening's fixed Settler quota.
        let alarm = self.conqueror_alarm(c);
        if (self.assessed_border_threat(c.pos)
            || self.threat_exposure(c) >= 2
            || (self.decisions.neighbour_alarm > 0 && alarm > self.local_defense(c)))
            && (!c.walls
                || (self.decisions.walled_defense_percent > 0 && self.defense_risk(c, None).falls)
                || alarm > self.local_defense(c))
        {
            if c.walls {
                let stake =
                    city_keep_value(c) * self.decisions.walled_defense_percent.max(alarm) / 100;
                let chosen = c
                    .detail
                    .as_ref()?
                    .options
                    .iter()
                    .filter_map(|o| {
                        let ProductionItem::Unit { unit } = o.item else {
                            return None;
                        };
                        (composition::is_melee_line(unit) || composition::is_ranged_line(unit))
                            .then_some((o, unit))
                    })
                    .max_by_key(|(o, unit)| {
                        let after = self.defense_risk(
                            c,
                            Some(crate::purchase_defense::purchased_defender(
                                self.view(),
                                c,
                                *unit,
                            )),
                        );
                        let stats = self
                            .view()
                            .rule_experiments
                            .unit_stats_for(self.faction, *unit);
                        ((if after.falls { 0 } else { stake })
                            + 2 * stats.strength.max(stats.ranged))
                            * 100
                            / o.turns.max(1)
                            - o.cost
                    })
                    .map(|(o, _)| o.item);
                return chosen.and_then(|item| {
                    self.early_choice(item, "anticipatory defense: walled city at risk")
                });
            }
            // A garrisoned shooter first (it fires every turn without retaliation and the
            // city shields it), then Walls, which end the rush era. Gene `walls_under_way`:
            // Walls already under way are finished, and a shooter two hexes out counts.
            let walls_kept = self.hard_defense() && self.decisions.walls_under_way > 0;
            if walls_kept
                && let Some(item @ ProductionItem::Building { building }) =
                    c.detail.as_ref().and_then(|d| d.producing)
                && data::building_family(building) == BuildingKind::Walls
            {
                return self.early_choice(item, "anticipatory defense: finish Walls under way");
            }
            let reach = if walls_kept { 2 } else { 1 };
            let shooter_here = self
                .ctx
                .own_units
                .iter()
                .any(|u| u.ranged > 0 && u.pos.distance(c.pos) <= reach);
            let walls = ProductionItem::Building {
                building: BuildingKind::Walls,
            };
            // Gene `swiss_playbook`: the Redoubt first; Gold buys the shooter if an attack
            // actually comes (`buy_emergency_defender`, door defense).
            let first = if shooter_here || self.walls_beat_arrival(c) || self.redoubt_garrison() {
                None
            } else {
                composition::RANGED_LADDER
                    .into_iter()
                    .map(|unit| ProductionItem::Unit { unit })
                    .find(|i| {
                        c.detail
                            .as_ref()
                            .is_some_and(|d| d.options.iter().any(|o| o.item == *i))
                    })
            };
            if let Some(item) = first.or_else(|| {
                c.detail
                    .as_ref()
                    .is_some_and(|d| d.options.iter().any(|o| o.item == walls))
                    .then_some(walls)
            }) {
                return self.early_choice(item, "anticipatory defense: shooter or Walls");
            }
            if self.faction == Faction::Mongols
                && shooter_here
                && self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| u.military && u.pos.distance(c.pos) <= 1)
                    .count()
                    < 2
                && let Some(item) = composition::MELEE_LADDER
                    .into_iter()
                    .map(|unit| ProductionItem::Unit { unit })
                    .find(|item| {
                        c.detail
                            .as_ref()
                            .is_some_and(|d| d.options.iter().any(|o| o.item == *item))
                    })
            {
                return self.early_choice(item, "anticipatory defense: Mongol second defender");
            }
            // Gene `neighbour_alarm`: with a shooter already here and no Walls to build, a
            // conqueror's alarm larger than the local defence still asks for a defender
            // rather than letting a Workshop or Market outvote it (Sparta 8.9.57: the
            // Mongols built Workshops at T38 to T42 with an alarm of 122 to 142 against a
            // local defence of 0 to 107; Almaliq fell at T59).
            if self.decisions.neighbour_alarm > 0
                && alarm > self.local_defense(c)
                && let Some(item) = c
                    .detail
                    .as_ref()
                    .and_then(|d| {
                        d.options
                            .iter()
                            .filter(|o| o.turns > 0)
                            .filter_map(|o| match o.item {
                                ProductionItem::Unit { unit }
                                    if composition::is_melee_line(unit)
                                        || composition::is_ranged_line(unit) =>
                                {
                                    Some((o, unit))
                                }
                                _ => None,
                            })
                            .max_by_key(|(o, unit)| {
                                let stats = self
                                    .view()
                                    .rule_experiments
                                    .unit_stats_for(self.faction, *unit);
                                (
                                    stats.strength.max(stats.ranged) * 100 / o.turns.max(1),
                                    std::cmp::Reverse(o.cost),
                                )
                            })
                    })
                    .map(|(o, _)| o.item)
            {
                return self.early_choice(item, "conqueror alarm: defender");
            }
        }
        if let Some(walls) = self.aztec_frontier_walls(c) {
            return self.early_choice(walls, "Aztec frontier Walls");
        }
        // A safe provincial court starts remitting while the opening still has time to
        // use the Gold. The ordinary opening queue would otherwise postpone the first
        // Revenue Office for many turns after Currency is known.
        if self.faction == Faction::India
            && self.difficulty == legatum_core::AiDifficulty::Hard
            && self.decisions.india_early_office > 0
            && self.view().turn <= 100
            && !c.is_capital
            && !self.funding_front(ci, c)
            && !self.assessed_border_threat(c.pos)
            && self.threat_exposure(c) == 0
            && let Some(d) = c.detail.as_ref()
            && !data::has_building(&d.buildings, BuildingKind::Market)
            && let Some(office) = d.options.iter().find(|o| {
                o.item
                    == ProductionItem::Building {
                        building: BuildingKind::Market,
                    }
                    && (o.yields.total.gold
                        + data::building_stats(BuildingKind::Market).yields.gold)
                        .max(0)
                        / 2
                        >= self.decisions.india_early_office_min_gold
            })
        {
            return self.early_choice(office.item, "India early Revenue Office");
        }
        // An exposed India city can repay a local opening army before Walls finish.
        // Otherwise send the first Settler before the ordinary field-army muster.
        // Gene `value_first_target`: the valued nomination replaces the faction-named prizes.
        let valued_prize = self.decisions.early_prize_opening > 0
            && self.value_first_target()
            && self.conquest_city().is_some_and(|enemy| {
                !enemy.walls
                    && self.ctx.own_cities.iter().any(|home| {
                        home.pos.distance(enemy.pos) <= self.decisions.open_prize_radius
                    })
            });
        let early_byzantium_prize = self.decisions.early_prize_opening > 0
            && !self.value_first_target()
            && self.view().cities.iter().any(|enemy| {
                !self.view().same_team(self.view().me, enemy.owner)
                    && self.view().faction_of(enemy.owner) == Faction::Byzantium
                    && !enemy.walls
                    && self.ctx.own_cities.iter().any(|home| {
                        home.pos.distance(enemy.pos) <= self.decisions.lone_city_strike_radius
                    })
            });
        let early_conquest_prize = self.decisions.early_prize_opening > 0
            && self.decisions.sparta_conquest_bundle > 0
            && self.view().cities.iter().any(|enemy| {
                !enemy.owner.is_barbarian()
                    && !self.view().same_team(self.view().me, enemy.owner)
                    && !enemy.walls
                    && self.ctx.own_cities.iter().any(|home| {
                        home.pos.distance(enemy.pos) <= self.decisions.open_prize_radius
                    })
            });
        let early_sparta_prize = early_byzantium_prize
            || early_conquest_prize
            || valued_prize
            || self.decisions.early_prize_opening > 0
                && !self.value_first_target()
                && self.view().cities.iter().any(|enemy| {
                    !self.view().same_team(self.view().me, enemy.owner)
                        && self.view().faction_of(enemy.owner) == Faction::India
                        && !enemy.walls
                        && self.ctx.own_cities.iter().any(|home| {
                            home.pos.distance(enemy.pos) <= self.decisions.open_prize_radius
                        })
                });
        // After the first capital falls, let the captured city make the first
        // Settler while the original capital replaces losses and fills the next
        // assault wing. Two simultaneous Settler queues strand the veterans.
        if self.decisions.early_prize_opening > 0
            && self.ctx.own_cities.len() == 2
            && self.ctx.own_cities[0].id == c.id
            && early_sparta_prize
            && tally.army < 8
            && self.view().turn <= self.decisions.early_prize_turn as u32
            && let Some(item) = composition::melee_first()
                .into_iter()
                .map(|unit| ProductionItem::Unit { unit })
                .find(|item| {
                    c.detail
                        .as_ref()
                        .is_some_and(|d| d.options.iter().any(|o| o.item == *item))
                })
        {
            return self.early_choice(item, "early prize: capital trains assault wing");
        }
        if self.decisions.early_prize_opening > 0
            && self.view().turn <= legatum_core::data::FIRST_THIRD_TURNS
            && (!early_sparta_prize
                || self.ctx.own_cities.len() == 1
                || self.ctx.own_cities[0].id == c.id)
            && (!self.first_settler_pending(c) || early_sparta_prize)
            && match self.wing_target().filter(|_| early_sparta_prize) {
                // Gene `assault_wing_forecast`: the wing is as big as the siege forecast says
                // taking the prize needs (playtest 8.9.2: the user trained a Hoplite every 4 to
                // 6 turns and stormed Nicaea with four; AI Sparta stopped at two and an Archer).
                Some(city) => !self.assault_wing_ready(city, None),
                None => {
                    tally.army
                        < self.ctx.own_cities.len() as i32 + if early_sparta_prize { 4 } else { 2 }
                }
            }
            && let Some(item) = (if early_byzantium_prize
                || early_conquest_prize
                || valued_prize
                // Gene `assault_wing_forecast`: the opening army is the assault wing, so it
                // is melee before any prize is seen (Hoplites ignore wounds and heal on kills).
                || self.decisions.assault_wing_forecast > 0 && self.hard_defense()
            {
                composition::melee_first()
            } else if tally.rangers == 0 && tally.army >= 2 {
                composition::ranged_first()
            } else {
                composition::melee_first()
            })
            .into_iter()
            .filter(|unit| !self.troop_unfunded(ci, c, *unit))
            .map(|unit| ProductionItem::Unit { unit })
            .find(|item| {
                c.detail
                    .as_ref()
                    .is_some_and(|d| d.options.iter().any(|o| o.item == *item))
            })
        {
            return self.early_choice(item, "early prize: train assault wing");
        }
        if self.rushing()
            && self.opening.is_some_and(|o| {
                o.next_build.is_none() && tally.army < o.policy.numbers.war_army + 2
            })
            && (self.view().economy.gold_per_turn.total >= data::military_upkeep(self.faction)
                || self.gold_left >= 60 * data::military_upkeep(self.faction))
        {
            // Keep a funded reinforcement on the way instead of switching the only
            // city to long-term investments while its first army is still besieging.
            let kinds = if tally.rangers < 2 {
                composition::ranged_then_soldier()
            } else {
                composition::MELEE_LADDER.to_vec()
            };
            if let Some(item) = kinds
                .into_iter()
                .filter(|unit| !self.troop_unfunded(ci, c, *unit))
                .map(|unit| ProductionItem::Unit { unit })
                .find(|item| {
                    c.detail
                        .as_ref()
                        .is_some_and(|d| d.options.iter().any(|o| o.item == *item))
                })
            {
                return self.early_choice(item, "rush: funded reinforcement");
            }
        }
        // Keep the merged Rush muster/reconnaissance ordering intact.
        if self.rushing()
            && let Some(item) = self.primitive_priority_production(c)
        {
            return self.early_choice(item, "rush primitive priority");
        }
        if self.faction == Faction::Rome && self.hard_defense() && self.decisions.roman_founders > 0
            && tally.army < (self.ctx.own_cities.len() as i32 * 2 + 1).max(5)
            && self.ctx.own_cities.len() < 6
            && let Some(item) = c.detail.as_ref()?.options.iter()
                .filter(|o| o.turns > 0 && o.resource_blocker.is_none()
                    && matches!(o.item, ProductionItem::Unit { unit } if legatum_core::data::unit_class(unit) == Some(legatum_core::UnitClass::Melee)))
                .max_by_key(|o| {
                    let ProductionItem::Unit { unit } = o.item else { unreachable!() };
                    let strength = legatum_core::data::unit_stats_for(Faction::Rome, unit).strength;
                    // The Legionary survives better and founds at population two.
                    strength * 10 + (if unit == UnitKind::Phalangite { 50 } else { 0 })
                        - o.turns * 4
                }).map(|o| o.item)
        {
            return self.early_choice(item, "Roman city value: train replacement founder");
        }
        if let Some(item) = self.opening_choice(c) {
            return self.early_choice(item, "opening choice");
        }
        if let Some(item) = self.primitive_priority_production(c) {
            return self.early_choice(item, "primitive priority");
        }
        if let Some(item) = self.pre_settler_build(c, tally) {
            return self.early_choice(item, "pre-Settler prerequisite");
        }
        let d = c.detail.as_ref()?;
        let mut best: Option<(i32, ProductionItem)> = None;
        for opt in &d.options {
            // Core normally removes faction locked items from this list. Keep the same
            // availability guard here so stale or synthetic views cannot score one through
            // the production jitter when its strategic value is zero.
            if !self.view().rule_experiments.available_to_player(
                self.faction,
                self.view().era_of(self.view().me),
                opt.item,
            ) {
                self.record_option_gate(opt.item, "faction or era availability");
                continue;
            }
            if matches!(opt.item, ProductionItem::Project { .. }) {
                self.record_option_gate(opt.item, "routine production skips Projects");
                continue;
            }
            if !self.opening_allows_optional_military(opt.item) {
                self.record_option_gate(opt.item, "opening forbids optional military");
                continue;
            }
            if let ProductionItem::Unit { unit } = opt.item
                && self.troop_unfunded(ci, c, unit)
            {
                self.record_option_gate(opt.item, "troop unfunded");
                continue;
            }
            if self.defer_slow_duplicate_unit(c, opt) {
                self.record_option_gate(opt.item, "faster city can finish this unit first");
                continue;
            }
            let value = (self.item_value(ci, c, opt, tally)
                + self.flower_wars_build_value(c, opt.item)
                - self.wonder_tile_penalty(opt))
                * self.strategy.map_or(100, |s| s.production_weight(opt.item))
                / 100;
            let primitive_bonus = self.primitive_production_bonus(c, opt);
            if value <= 0 && primitive_bonus <= 0 {
                self.record_option_gate(opt.item, "nonpositive item value before time discount");
                continue;
            }
            let horizon = 60;
            let item_key = match opt.item {
                ProductionItem::Unit { unit } => unit as u64,
                ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
                    128 + building as u64
                }
                ProductionItem::Improvement { improvement, .. } => 192 + improvement as u64,
                ProductionItem::Project { .. } => unreachable!(),
            };
            // Finish what was started: stored progress (the build under way's own, or a bank) is
            // only worth anything to this item, so a nearly done build outranks an equal fresh
            // one (up to +50%). The view zeroes `production_banked` for the build under way.
            let banked = value.max(0) * opt.production_stored.clamp(0, opt.cost.max(1))
                / opt.cost.max(1)
                / 2;
            // Rank the first Settler at its full price: its discount only offsets the build
            // a population-1 capital makes first, so it must not crowd out an exposed
            // capital's defender or other priorities tuned before the discount.
            let turns = if opt.item
                == (ProductionItem::Unit {
                    unit: UnitKind::Settler,
                })
                && self.first_settler_pending(c)
                && opt.turns > 0
            {
                let full = opt.cost * 100 / data::FIRST_SETTLER_PERCENT;
                let rate = opt.yields.total.production.max(1);
                ((full - opt.production_stored).max(1) + rate - 1) / rate
            } else {
                opt.turns
            };
            let score = value.max(0) * horizon / (turns.clamp(0, 400) + horizon)
                + banked
                + primitive_bonus
                + self.jitter(tag::PRODUCTION, (c.id.0 as u64) << 8 | item_key, 15);
            self.record_option(opt.item, value, score);
            if best.is_none_or(|(s, _)| score > s) {
                best = Some((score, opt.item));
            }
        }
        best.map(|(_, item)| item).or_else(|| {
            (self.faction == Faction::Germany)
                .then(|| ProductionItem::Project {
                    project: if self.view().economy.gold_per_turn.total <= 0 {
                        legatum_core::IndustryProject::Gold
                    } else {
                        legatum_core::IndustryProject::Science
                    },
                })
                .filter(|item| d.options.iter().any(|option| option.item == *item))
        })
    }

    fn defer_slow_duplicate_unit(&self, c: &CityView, opt: &ProductionOption) -> bool {
        let ProductionItem::Unit { unit } = opt.item else {
            return false;
        };
        let window = self.decisions.unit_concentration_turns;
        if self.difficulty != AiDifficulty::Hard
            || window == 0
            || !composition::is_combat(unit)
            || opt.turns <= window
            || c.detail
                .as_ref()
                .is_some_and(|d| d.producing == Some(opt.item))
        {
            return false;
        }
        self.ctx.own_cities.iter().any(|other| {
            if other.id == c.id {
                return false;
            }
            let Some(d) = &other.detail else {
                return false;
            };
            if d.resistance > 0
                || !(d.producing.is_none()
                    || d.producing == Some(opt.item)
                    || matches!(d.producing, Some(ProductionItem::Project { .. }))
                    || self.orders.iter().any(|order| {
                        matches!(order, Order::SetProduction { city, item }
                            if *city == other.id && *item == opt.item)
                    }))
            {
                return false;
            }
            d.options.iter().any(|candidate| {
                candidate.item == opt.item && (0..opt.turns).contains(&candidate.turns)
            })
        })
    }

    /// Site selection is a real production trade: preserve valuable worked land and keep the
    /// visible, transferable points away from an exposed border.
    pub(crate) fn wonder_tile_penalty(&self, opt: &ProductionOption) -> i32 {
        let Some(site) = opt.wonder.as_ref() else {
            return 0;
        };
        let lost = site.lost;
        let edges = site
            .tile
            .neighbors()
            .into_iter()
            .filter(|h| {
                self.view()
                    .tile(*h)
                    .is_none_or(|t| t.owner != Some(self.ctx.me))
            })
            .count() as i32;
        let rival = self
            .ctx
            .enemy_cities
            .iter()
            .map(|c| c.pos.distance(site.tile))
            .chain(
                self.ctx
                    .enemy_units
                    .iter()
                    .filter(|u| u.military)
                    .map(|u| u.pos.distance(site.tile)),
            )
            .min()
            .unwrap_or(12);
        lost.food * 30
            + lost.production * 40
            + lost.gold * 20
            + lost.science * 40
            + edges * 12
            + (8 - rival).max(0) * 10
    }

    /// Opportunity cost in combat points per 1,000 Gold. The old macro military
    /// score of 2,400 corresponds to the strength-20 reference's 20 combat points.
    pub(crate) fn modernization_alternative(&self) -> i64 {
        let mut tally = Tally {
            army: 0,
            rangers: 0,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: self.count_good_sites(4),
        };
        for u in &self.ctx.own_units {
            tally.add(ProductionItem::Unit { unit: u.kind });
        }
        self.ctx
            .own_cities
            .iter()
            .enumerate()
            .flat_map(|(ci, c)| {
                c.detail
                    .as_ref()
                    .into_iter()
                    .flat_map(|d| &d.options)
                    .filter_map(|o| {
                        let cost = o.buy_cost.filter(|cost| *cost > 0)?;
                        if o.buy_block.is_some() || o.resource_blocker.is_some() {
                            return None;
                        }
                        let value = match o.item {
                            ProductionItem::Unit { unit } if composition::is_combat(unit) => self
                                .modernization_power(
                                    unit,
                                    u8::from(self.faction == Faction::Sparta),
                                    data::UNIT_HP,
                                    false,
                                    c.pos,
                                ),
                            _ => self.item_value(ci, c, o, &tally).max(0) / 120,
                        };
                        Some(i64::from(value) * 1000 / i64::from(cost))
                    })
                    .collect::<Vec<_>>()
            })
            .max()
            .unwrap_or(0)
    }

    fn item_value(&self, ci: usize, c: &CityView, opt: &ProductionOption, t: &Tally) -> i32 {
        if self.faction == Faction::Egypt
            && self.difficulty == AiDifficulty::Hard
            && self.decisions.egypt_food_buildings > 0
            && self.view().rule_experiments.egypt_construction()
            && matches!(opt.item, ProductionItem::Building { .. })
        {
            let detail = c.detail.as_ref().expect("own city detail");
            let surplus = opt.yields.total.food - 2 * c.population;
            if surplus <= 0 {
                return 0;
            }
            // After the first Granary, let a small city train or expand while it grows
            // toward the next population gate. The option's turns already use Food.
            if c.population < 3
                && data::has_building(&detail.buildings, BuildingKind::Granary)
                && opt.item
                    != (ProductionItem::Building {
                        building: BuildingKind::Walls,
                    })
            {
                return 0;
            }
        }
        if self.opening.is_some()
            && opt.item
                == (ProductionItem::Unit {
                    unit: UnitKind::Settler,
                })
        {
            // Opening Settlers are selected explicitly, never traded off against the
            // time discount or made ahead of the policy's timing/target by routine scoring.
            return 0;
        }
        let weight = match opt.item {
            ProductionItem::Unit {
                unit: UnitKind::Settler,
            } => self.decisions.production_settler_percent,
            ProductionItem::Unit { unit }
                if self
                    .view()
                    .rule_experiments
                    .unit_stats_for(self.faction, unit)
                    .military =>
            {
                self.decisions.production_military_percent
            }
            ProductionItem::Building { .. } | ProductionItem::Wonder { .. } => {
                self.decisions.production_building_percent
            }
            ProductionItem::Project { .. } => 0,
            _ => 100,
        };
        let combat_percent = match opt.item {
            ProductionItem::Unit { unit }
                if composition::is_combat(unit) && self.decisions.modernization_value > 0 =>
            {
                (100 + (self.modernization_percent(unit, c.pos) - 100)
                    * self.decisions.modernization_value
                    / 100)
                    .max(0)
            }
            _ => 100,
        };
        let mut modern_tally = *t;
        if self.decisions.modernization_value > 0 {
            // A full head-count of obsolete troops must not close the replacement budget.
            // Keep queued units in the tally; only discount the existing field army.
            let mut army_hundredths = 0;
            let mut ranged_hundredths = 0;
            for u in &self.ctx.own_units {
                if composition::is_combat(u.kind) {
                    let percent = self.modernization_percent(u.kind, c.pos).min(100);
                    army_hundredths += percent;
                    modern_tally.army -= 1;
                    if composition::is_ranged_line(u.kind) {
                        modern_tally.rangers -= 1;
                        ranged_hundredths += percent;
                    }
                }
            }
            modern_tally.army += (army_hundredths + 99) / 100;
            modern_tally.rangers += (ranged_hundredths + 99) / 100;
        }
        let value = self.item_base_value(ci, c, opt, &modern_tally) * combat_percent / 100 * weight
            / 100
            * (100 + self.objective_item_bonus(c, opt.item))
            / 100
            + self.objective_build_value(c, opt.item)
            - self.german_industry_food_cost(c, opt);
        // Apply the shared shooter need after modernization and other general
        // weights. A base-value bonus can be discounted below the melee quota,
        // and a quota can give a needed shooter zero, recreating the
        // emergency/routine alternation even though both saw the same threat.
        let value = match opt.item {
            ProductionItem::Unit { unit }
                if self.difficulty == AiDifficulty::Hard
                    && self.decisions.production_threat_agreement > 0
                    && composition::is_ranged_line(unit)
                    && self.ranged_relief_needed(c).1 =>
            {
                value.max(4200 + composition::quality(unit))
            }
            _ => value,
        };
        // Another Iron or Coal user can crowd out the remaining Foundry chain.
        // Price its opportunity cost; a valuable defender can still outbid it.
        let supply_cost = match legatum_core::resources::item_need(opt.item) {
            Some(resource)
                if c.detail
                    .as_ref()
                    .is_some_and(|d| d.producing != Some(opt.item))
                    && !matches!(
                        opt.item,
                        ProductionItem::Building {
                            building: BuildingKind::Forge | BuildingKind::Factory
                        }
                    ) =>
            {
                self.factory_goal_supply_value(resource)
            }
            _ => 0,
        };
        (value - supply_cost).max(0)
    }

    fn german_industry_food_cost(&self, c: &CityView, opt: &ProductionOption) -> i32 {
        let (ProductionItem::Building { building } | ProductionItem::Wonder { building, .. }) =
            opt.item
        else {
            return 0;
        };
        if self.faction != Faction::Germany {
            return 0;
        }
        if self
            .view()
            .rule_experiments
            .faction_rules_off
            .contains(legatum_core::experiments::FactionRule::GermanyIndustryFood)
        {
            return 0;
        }
        if self.difficulty != AiDifficulty::Hard || self.decisions.german_industry_food == 0 {
            return 0;
        }
        if !data::is_production_building(building) {
            return 0;
        }
        let surplus = opt.yields.total.food - 2 * c.population;
        if c.population <= 3 && surplus <= 1 {
            return 10_000;
        }
        let scarcity = if surplus <= 0 {
            4
        } else if surplus <= 2 {
            2
        } else {
            1
        };
        scarcity * (100 + self.knobs.expansion * 2)
    }

    fn legacy_foundry_food_cost(&self, building: BuildingKind) -> i32 {
        if self.faction == Faction::Germany
            && building == BuildingKind::Forge
            && !self
                .view()
                .rule_experiments
                .faction_rules_off
                .contains(legatum_core::experiments::FactionRule::GermanyIndustryFood)
            && (self.difficulty != AiDifficulty::Hard || self.decisions.german_industry_food == 0)
        {
            // Gene 0 restores the old Foundry price at its old point in item_base_value,
            // before the item_value multipliers. The new rule itself still applies in play.
            2 * (100 + self.knobs.expansion * 2)
        } else {
            0
        }
    }

    fn item_base_value(&self, ci: usize, c: &CityView, opt: &ProductionOption, t: &Tally) -> i32 {
        let k = self.knobs;
        let ctx = &self.ctx;
        let d = c.detail.as_ref().expect("own city detail");
        // Barbarians never assault cities: a raid doesn't change what a city builds.
        let threat = ctx.threat[ci] - ctx.barbarian_threat[ci];
        let threatened = threat > 0;
        let undefended = threat > self.local_defense(c);
        let cities = ctx.own_cities.len() as i32;
        let solvent = self.view().economy.gold_per_turn.total > 0 || t.army < 2;
        let has = |b: BuildingKind| {
            legatum_core::data::has_building(&d.buildings, b)
                || self.orders.iter().any(|o| {
                    matches!(o,
                    Order::BuyItem { city, item: ProductionItem::Building { building } }
                        if *city == c.id && legatum_core::data::building_satisfies(*building, b))
                })
        };
        let early = self.view().turn < self.decisions.opening_chain_turn as u32;
        let peaceful_opening_chain = self.difficulty == legatum_core::AiDifficulty::Hard
            && self.decisions.settler_chain > 0
            && c.is_capital
            && cities < 4
            && has(BuildingKind::Granary)
            && !threatened
            && !self.last_city_front(c)
            && !self.ctx.threat.iter().any(|threat| *threat > 0);
        if self.strategy == Some(crate::Strategy::Rush)
            && self.view().turn <= self.decisions.rush_window_turn as u32
        {
            if opt.item
                == (ProductionItem::Unit {
                    unit: UnitKind::Settler,
                })
            {
                return 0;
            }
            if !threatened && let Some(item) = self.rush_opening_item(c, t) {
                return if opt.item == item { 10000 } else { 0 };
            }
        }
        match opt.item {
            ProductionItem::Project { .. } => 0,
            ProductionItem::Improvement { .. } => {
                if threatened {
                    0
                } else {
                    let food_support = peaceful_opening_chain
                        && c.population < 2
                        && opt.improvement.as_ref().is_some_and(|i| i.gain.food > 0);
                    self.improvement_value(opt)
                        + if food_support {
                            self.decisions.settler_chain_priority * 3
                        } else {
                            0
                        }
                }
            }
            ProductionItem::Unit { unit } => match unit {
                UnitKind::Settler => {
                    // A nearby war over an exposed city has first claim on production.
                    // Founding can resume once the local siege is resolved.
                    let local_siege = self.last_city_front(c)
                        // The threat pass already interrupts a civilian for a
                        // visible approach. Do not resume it in routine
                        // scoring while that same exposure still holds.
                        || (self.opening.is_none() && self.visible_threat_approach(c))
                        || self.ctx.enemy_cities.iter().any(|enemy| {
                            self.view().at_war(self.view().me, enemy.owner)
                                && enemy.visible
                                && enemy.hp < enemy.max_hp
                                && c.pos.distance(enemy.pos) <= 8
                                && self
                                    .ctx
                                    .own_units
                                    .iter()
                                    .any(|u| u.military && u.pos.distance(enemy.pos) <= 4)
                        })
                        || self.ctx.own_cities.iter().any(|front| {
                            front.id != c.id
                                && front.hp * 2 <= front.max_hp
                                && c.pos.distance(front.pos) <= 8
                                && (front.detail.as_ref().is_some_and(|d| d.resistance > 0)
                                    || self.ctx.enemy_units.iter().any(|enemy| {
                                        enemy.military
                                            && !enemy.owner.is_barbarian()
                                            && enemy.pos.distance(front.pos) <= 6
                                    }))
                        });
                    let pop_ok =
                        c.population >= 2 || d.turns_to_grow.is_some_and(|g| g <= opt.turns);
                    // How many cities to want is a value question, not a quota: land (room,
                    // sites), time left and the rising price decide when expansion stops.
                    // Opt-in strategy personalities and timid Easy keep their city targets.
                    let want = match self.strategy {
                        Some(s) => cities + t.settler_pipeline < s.target_cities(),
                        None if self.difficulty == legatum_core::AiDifficulty::Easy => {
                            cities + t.settler_pipeline < k.target_cities()
                        }
                        None => true,
                    } && self.wild_allows_settling();
                    // Production, a short journey, founding, then a few turns to contribute.
                    // Do not consume population on a settler that cannot repay it this game.
                    let time = opt.turns >= 0 && opt.turns + 24 < self.turns_remaining();
                    // Any good site in the empire's reach counts, not only this city's
                    // surroundings: the core city trains the Settler for the frontier.
                    let best_site = self
                        .ctx
                        .site_score
                        .iter()
                        .enumerate()
                        .filter_map(|(i, score)| {
                            let h = self.ctx.hex_at(i);
                            score
                                .filter(|s| *s >= self.decisions.settler_min_site_score)
                                .filter(|_| {
                                    self.ctx.own_cities.iter().any(|home| {
                                        home.pos.distance(h)
                                            <= self.decisions.settler_max_site_distance
                                    }) && self.ctx.reachable(c.pos, h)
                                        && !self.ctx.in_danger(h)
                                })
                        })
                        .max();
                    let room = best_site.is_some();
                    // Grow, invest, then export one citizen. The first colony is urgent;
                    // later settlers come from developed cities instead of resetting every
                    // new settlement to population one. Count the full pipeline.
                    let developed = self.first_settler_pending(c)
                        || (has(BuildingKind::Workshop)
                            && (c.population >= 3 || has(BuildingKind::Granary)))
                        // A grown city exports a citizen without waiting for a Workshop.
                        || c.population >= 4
                        || (!has(BuildingKind::Granary)
                            && !has(BuildingKind::Workshop)
                            && d.options.iter().all(|o| {
                                !matches!(
                                    o.item,
                                    ProductionItem::Building {
                                        building: BuildingKind::Granary | BuildingKind::Workshop
                                    }
                                )
                            }));
                    // Hard claims its first four sites before developing each colony;
                    // defended garrisons keep those young cities from feeding a rusher.
                    let hard = self.difficulty == legatum_core::AiDifficulty::Hard;
                    let pipeline_limit = if hard {
                        self.decisions.settler_pipeline
                    } else if cities < 4 {
                        1
                    } else {
                        2
                    };
                    let developed = developed
                        || (hard
                            && cities + t.settler_pipeline < self.decisions.settler_open_cities);
                    // With the colony priced (gene `colony_compound`), one defender is the
                    // floor and a second troop has to outbid the Settler on value.
                    let army_floor = if hard && self.decisions.colony_compound > 0 {
                        1
                    } else {
                        2
                    };
                    let army_ok = !early || t.army >= army_floor || self.first_settler_pending(c);
                    // Gene `settler_slow_turns`: other cities' slow Settlers leave the gate's
                    // pipeline count; this city's own Settler under way always stays in it.
                    let own_slow = i32::from(
                        self.decisions.settler_slow_turns > 0
                            && item_is_settler(d.producing)
                            && opt.turns >= self.decisions.settler_slow_turns,
                    );
                    let mut gates = SettlerGates {
                        threatened,
                        pop_ok,
                        want,
                        time,
                        room,
                        developed,
                        army_ok,
                        sites_ok: t.sites > t.settler_pipeline,
                        pipeline_ok: t.settler_pipeline - t.slow_settlers + own_slow
                            < pipeline_limit,
                        value: 0,
                    };
                    let value = if !threatened
                        && !local_siege
                        && pop_ok
                        && want
                        && time
                        && room
                        && developed
                        && army_ok
                        && gates.sites_ok
                        && gates.pipeline_ok
                    {
                        // The rising settler cost is already in opt.turns; also discount it.
                        let base = 650 + k.expansion * 6 - (opt.cost / 2 - 80);
                        if self.first_settler_pending(c) {
                            base + if self.difficulty == legatum_core::AiDifficulty::Hard
                                && !self.opening_exposed(c)
                            {
                                1100
                            } else {
                                400
                            }
                        } else {
                            (base + 1400 + k.expansion * 10)
                                * self.colony_payback_percent(opt.turns)
                                / 100
                        }
                    } else {
                        0
                    };
                    // Hard lifts a routine Settler over the army and building quotas it would
                    // otherwise tie with (gene `settler_priority`); the first Settler already
                    // outranks them.
                    let at_peace = !self.view().players.iter().any(|p| {
                        p.alive
                            && !self.view().same_team(self.view().me, p.id)
                            && self.view().at_war(self.view().me, p.id)
                    });
                    let value = if value > 0 && hard && at_peace && !self.first_settler_pending(c) {
                        value + self.decisions.settler_priority
                    } else {
                        value
                    };
                    // In a quiet opening, keep the capital's Settler queue ahead of routine
                    // military and infrastructure quotas. The existing threat assessment and
                    // siege gates above remain authoritative; Walls and emergency defence are
                    // still selected through their normal production paths when danger appears.
                    let quiet_chain = peaceful_opening_chain
                        && t.settler_pipeline < self.decisions.settler_pipeline
                        && !local_siege;
                    let value = if value > 0 && quiet_chain {
                        value + self.decisions.settler_chain_priority
                    } else {
                        value
                    };
                    let value = if value > 0 {
                        value + self.colony_compound_value(c, opt, best_site.unwrap_or(0), cities)
                    } else {
                        value
                    };
                    // Before five Foundry cities, expansion helps Germany's goal.
                    // The negative cost adds a bounded value to ordinary settlement.
                    let value = (value - self.factory_goal_expansion_cost()).max(0);
                    gates.value = value;
                    self.record_settler(gates);
                    value
                }
                UnitKind::Scout => {
                    let explore = ctx.frontier.iter().any(|h| ctx.reachable(c.pos, *h))
                        || ctx
                            .enemy_cities
                            .iter()
                            .any(|e| !e.visible && ctx.reachable(c.pos, e.pos));
                    let information: i32 = self
                        .view()
                        .players
                        .iter()
                        .filter(|p| p.alive && !self.view().same_team(self.view().me, p.id))
                        .map(|p| self.information_value(p.id))
                        .max()
                        .unwrap_or(0);
                    if t.scouts == 0
                        && (explore || information >= data::unit_stats(UnitKind::Scout).cost)
                        && !threatened
                    {
                        200 + self.decisions.scout_priority_bonus + information
                    } else {
                        0
                    }
                }
                UnitKind::HeirToTheThrone if t.heir_pipeline == 0 => {
                    self.heir_value(c, opt.turns, threatened, undefended)
                }
                UnitKind::HeirToTheThrone => 0,
                unit if data::is_aircraft(unit)
                    && self.difficulty != legatum_core::AiDifficulty::Easy =>
                {
                    if threatened && undefended {
                        0
                    } else {
                        let air_value = crate::air::production_value_with_prizes(
                            self.view(),
                            c,
                            unit,
                            self.decisions.air_defense,
                            &self.orders,
                            self.decisions.air_stage_read != 0
                                && self.difficulty == legatum_core::AiDifficulty::Hard,
                        );
                        let oil_alternative = crate::supply_value::aircraft_oil_opportunity(
                            self.view(),
                            self.decisions.supply_war_percent,
                        );
                        (air_value - self.decisions.industry_priority * oil_alternative).max(0)
                    }
                }
                unit if composition::is_combat(unit) => {
                    // A Pikeman takes the Iron a Forge-ready city is waiting for unless its
                    // strength is worth more than the Production the Forge would add.
                    if let Some(iron) = legatum_core::resources::unit_need(unit)
                        && let Some(from) = composition::MELEE_LADDER
                            .into_iter()
                            .find(|f| data::unit_stats(*f).upgrades_to == Some(unit))
                        && !crate::supply_value::unit_outbids_industry(
                            self.view(),
                            from,
                            unit,
                            iron,
                            self.decisions.supply_war_percent,
                            self.decisions.mongol_horde >= 2 && self.mongol_horde(),
                        )
                    {
                        return 0;
                    }
                    // Gene `celts_playbook` part 1: sheltered Celtic cities need no garrison.
                    let guarded = self.garrisoned_city_count();
                    if self.campaigns.plan.posture == Some(crate::plan::Posture::Build)
                        && self.ctx.threat[ci] == 0
                        && t.army >= guarded
                    {
                        return 0;
                    }
                    if self.decisions.modernization_value == 0
                        && composition::obsolete(unit, |t| self.view().research.known.contains(&t))
                    {
                        return 0;
                    }
                    let mounted = self.mongol_horde()
                        && composition::is_mounted(unit)
                        && self.best_mounted_option(c) == Some(unit);
                    let counter = if mounted {
                        self.early_counter_value(c, UnitKind::Horseman)
                            .max(self.early_counter_value(c, UnitKind::Spearman))
                    } else if self.mongol_horde() {
                        0
                    } else {
                        self.early_counter_value(c, unit)
                    };
                    if counter > 0 && solvent {
                        return 2600 + counter + composition::quality(unit);
                    }
                    if (self.conquest_city().is_some() || self.contain_war().is_some())
                        && solvent
                        && t.army < self.desired_army()
                    {
                        let current = self.upkeep_unit_count();
                        let extra_upkeep =
                            self.military_upkeep_at(if self.decisions.modernization_value > 0 {
                                current + 1
                            } else {
                                t.army + 1
                            }) - self.military_upkeep_at(current);
                        let deficit =
                            (extra_upkeep - self.view().economy.gold_per_turn.total).max(0);
                        if t.army >= cities
                            && deficit * 30 > (self.gold_left - self.gold_reserve()).max(0)
                        {
                            return 0;
                        }
                        let ranged = composition::is_ranged_line(unit);
                        let need_ranged = t.rangers < 3 && t.army - t.rangers > cities;
                        if mounted {
                            return 2500 + composition::quality(unit);
                        }
                        if !self.mongol_horde() && ranged == need_ranged {
                            // The class triangle picks between melee and mounted here too.
                            return 2400
                                + composition::quality(unit)
                                + self.composition_bonus(unit);
                        }
                    }
                    if self.strategy == Some(crate::Strategy::Rush)
                        && !threatened
                        && t.army >= cities + 2
                        && t.rangers * 3 < t.army
                        && !composition::is_ranged_line(unit)
                        && d.options.iter().any(|o| {
                            matches!(
                                o.item,
                                ProductionItem::Unit { unit } if composition::is_ranged_line(unit)
                            )
                        })
                    {
                        // Leave room in the affordable quota for ranged support; a
                        // third cheap melee unit must not crowd it out indefinitely.
                        return 0;
                    }
                    // A searched build-ratio posture moves the ranged share (a third by default).
                    let ranged_divisor =
                        crate::plan::Posture::ranged_divisor(self.campaigns.plan.posture);
                    let kind_bonus = self.unique_unit_bonus(unit)
                        + self.shrine_training_bonus(c, unit)
                        + self.composition_bonus(unit)
                        + match unit {
                            UnitKind::Pikeman => 150,
                            UnitKind::Musketman => 380,
                            UnitKind::Rifleman => 510,
                            UnitKind::TrenchInfantry => 575,
                            UnitKind::Infantry => 640,
                            UnitKind::MechanisedInfantry => 700,
                            UnitKind::HunterKiller => 770,
                            // The advanced shooters: a share of the army.
                            ranged
                                if composition::is_ranged_line(ranged)
                                    && ranged != UnitKind::Archer
                                    && t.rangers * ranged_divisor < t.army + 1 =>
                            {
                                350 + k.military
                            }
                            ranged
                                if composition::is_ranged_line(ranged)
                                    && ranged != UnitKind::Archer =>
                            {
                                if threatened {
                                    160
                                } else {
                                    -40
                                }
                            }
                            // Archers defend: favored under threat and as a third of the army.
                            UnitKind::Archer if threatened => 100,
                            UnitKind::Archer if t.rangers * ranged_divisor < t.army + 1 => 180,
                            UnitKind::Archer => -40,
                            _ => 0,
                        };
                    // Ranged support can answer the bombardment that city strength cannot.
                    // One melee garrison plus two shooters is a useful force; repeatedly
                    // choosing the cheaper Spearman fills a numerical quota without it.
                    if !self.mongol_horde()
                        && self.difficulty == legatum_core::AiDifficulty::Hard
                        && t.army > t.rangers
                        && t.rangers < ((guarded + 1) / 2).min(2)
                        && (cities >= 2
                            || !self.first_settler_pending(c)
                            || self.opening_exposed(c))
                        && solvent
                        && composition::is_ranged_line(unit)
                    {
                        return 3300 + kind_bonus;
                    }
                    // An army target that always loses to another overdue building is
                    // not a garrison minimum. Fund the missing defenders before optional
                    // infrastructure, including troops already in city queues.
                    // Gene `swiss_playbook`: walled cities need no troop; the two free
                    // units stay the floor. Gene `celts_playbook` part 1: sheltered Celtic
                    // cities need none either (`guarded`).
                    let minimum = if self.decisions.walls_garrison_value > 0 {
                        self.troop_garrison_cities()
                            .min(guarded)
                            .max(data::FREE_MILITARY_UNITS)
                    } else if self.redoubt_garrison() {
                        self.troop_garrison_cities().max(data::FREE_MILITARY_UNITS)
                    } else {
                        guarded
                            + i32::from(
                                self.difficulty == legatum_core::AiDifficulty::Hard && cities >= 3,
                            )
                    };
                    if k.military > 0
                        && (t.army < minimum
                            || self.heir_unguarded(c)
                            || self.hall_unguarded(c)
                            || ((self.wonder_guard_city(c) || self.own_original_capital(c))
                                && !self
                                    .ctx
                                    .own_units
                                    .iter()
                                    .any(|u| u.military && u.pos == c.pos)))
                        && solvent
                    {
                        // The capitals victory: a pressed, empty capital outranks every
                        // other city's build.
                        if self.capital_pressed(c)
                            && !self
                                .ctx
                                .own_units
                                .iter()
                                .any(|u| u.military && u.pos == c.pos)
                        {
                            return CAPITAL_PRESSED_VALUE + kind_bonus;
                        }
                        if self.mongol_horde() && threatened && !composition::is_melee_line(unit) {
                            return 0;
                        }
                        return GARRISON_FLOOR_VALUE + kind_bonus;
                    }
                    if !self.mongol_horde()
                        && self.difficulty == legatum_core::AiDifficulty::Hard
                        && cities >= 2
                        && t.rangers == 0
                        && solvent
                        && self
                            .ctx
                            .own_cities
                            .iter()
                            .any(|city| self.exposed_city(city))
                        && composition::is_ranged_line(unit)
                    {
                        return 1900 + kind_bonus;
                    }
                    if early
                        && t.army < 2
                        && unit == UnitKind::Spearman
                        && (self.opening_exposed(c) || !self.first_settler_pending(c))
                    {
                        k.opening_defense_value()
                            .max(if undefended { 1000 } else { 0 })
                            + kind_bonus
                    } else if undefended {
                        1000 + kind_bonus
                    } else if early
                        && self.strategy != Some(crate::Strategy::Rush)
                        && self.first_settler_pending(c)
                        && (t.army >= 2 || !self.opening_exposed(c))
                    {
                        // The starting garrison plus one trained Spearman protect expansion.
                        0
                    } else if t.army < self.desired_army() && solvent {
                        200 + k.military * 5 + kind_bonus + self.decisions.army_item_bonus
                    } else {
                        0
                    }
                }
                _ => 0,
            },
            ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
                if has(building) {
                    return 0;
                }
                self.infrastructure_bonus(ci, c, building)
                    + self.goal_building_bonus(c, building, opt.turns)
                    + if !threatened {
                        self.work_bonus(c.id, building)
                    } else {
                        0
                    }
                    + match building {
                        BuildingKind::Mill | BuildingKind::Supermarket => {
                            if d.yields.total.food - 2 * c.population < 4 {
                                180 + k.economy * 3 + k.expansion
                            } else {
                                50 + k.economy
                            }
                        }
                        BuildingKind::Forge | BuildingKind::Factory => {
                            // Industry must earn back its construction before it can fund
                            // the next investment. Use the same 30-turn planning horizon
                            // as choose_item, and the quoted marginal yield (including
                            // India's distance penalty), rather than a flat upgrade bonus.
                            // plan_victory can still buy production for an immediate title.
                            let gain = self
                                .view()
                                .research
                                .unlocks
                                .iter()
                                .find(|u| u.item == opt.item)
                                .and_then(|u| u.cities.iter().find(|u| u.city == c.id))
                                .map_or_else(
                                    || {
                                        self.view()
                                            .rule_experiments
                                            .building_stats_for(self.faction, building)
                                            .yields
                                            .production
                                    },
                                    |u| {
                                        (u.yields_after.total.production
                                            - opt.yields.total.production)
                                            .max(0)
                                    },
                                );
                            // The supply it uses is spare (an option without it is removed), so
                            // its yield costs no one anything: priority by Production gained.
                            // A city whose growth has stalled feeds itself first (the same
                            // test as Mill).
                            let supply = if threatened || d.yields.total.food - 2 * c.population < 4
                            {
                                0
                            } else {
                                self.decisions.industry_priority * gain
                            };
                            (180 + k.economy * 4 + k.military) * gain * 60 / opt.cost.max(1)
                                + supply
                                - self.legacy_foundry_food_cost(building)
                        }
                        BuildingKind::University => 140 + k.science * 5,
                        BuildingKind::ResearchLab => {
                            let window = 40;
                            if opt.turns >= 0 && opt.turns + window <= self.turns_remaining() {
                                200 + k.science * 6
                            } else {
                                0
                            }
                        }
                        BuildingKind::Bank | BuildingKind::StockExchange => {
                            120 + k.economy * 4 + if !solvent { 300 } else { 0 }
                        }
                        BuildingKind::Agoge => {
                            (if self.decisions.war_shrine_drive > 0
                                && self.desired_army()
                                    > ctx.own_units.iter().filter(|u| u.military).count() as i32
                            {
                                240 + k.military * 3
                            } else {
                                40
                            }) + self.shrine_value(c)
                        }
                        BuildingKind::Castle | BuildingKind::StarFort => {
                            if threatened {
                                500 + k.military
                            } else {
                                20
                            }
                        }
                        // The Rice Storehouse stays with Japan when it becomes playable again.
                        BuildingKind::RiceStorehouse => 0,
                        // Valued with the Modern era's deliberate air-defence play (increment 3).
                        BuildingKind::AirDefence => 0,
                        BuildingKind::Granary => 100 + k.economy * 2 + k.expansion,
                        BuildingKind::Workshop => 100 + k.economy * 2 + k.military,
                        BuildingKind::Market => {
                            let toll = if self.faction == Faction::Switzerland
                                && !self.rule_on(legatum_core::experiments::FactionRule::SwissTolls)
                            {
                                0
                            } else {
                                self.decisions.toll_market_bonus
                            };
                            let remittance = if self.faction == Faction::India
                                && !c.is_capital
                                && self.rule_on(
                                    legatum_core::experiments::FactionRule::IndiaRemittance,
                                ) {
                                // An option previews its production focus before the building
                                // completes. Add the Office's +3 to price the remittance it unlocks.
                                let weight = if self.difficulty == legatum_core::AiDifficulty::Hard
                                {
                                    self.decisions.india_remittance_value
                                } else {
                                    70
                                };
                                (opt.yields.total.gold
                                    + data::building_stats(BuildingKind::Market).yields.gold)
                                    .max(0)
                                    / 2
                                    * weight
                            } else {
                                0
                            };
                            let goal = if self.faction == Faction::India
                                && self.difficulty == legatum_core::AiDifficulty::Hard
                                && self.goal_pending()
                                && self.view().turn >= 60
                            {
                                let missing = self
                                    .ctx
                                    .own_cities
                                    .iter()
                                    .filter(|city| {
                                        city.detail.as_ref().is_some_and(|d| {
                                            !data::has_building(&d.buildings, BuildingKind::Market)
                                        })
                                    })
                                    .count() as i32;
                                self.decisions.india_office_goal_value
                                    + if (1..=2).contains(&missing) {
                                        self.decisions.india_last_office_value / missing
                                    } else {
                                        0
                                    }
                            } else {
                                0
                            };
                            80 + k.economy * 3
                                + toll
                                + self.swiss_market_priority()
                                + remittance
                                + goal
                        }
                        BuildingKind::Library => 80 + k.science * 4,
                        BuildingKind::YamRelayStation => self.hall_value(c, opt.turns),
                        BuildingKind::ImperialDiwan => {
                            if self.faction == Faction::India && self.goal_pending() {
                                1200 + k.economy * 4
                            } else {
                                0
                            }
                        }
                        BuildingKind::Roadworks => self.roadworks_value(c),
                        BuildingKind::Walls if self.capital_pressed(c) => {
                            CAPITAL_PRESSED_VALUE + k.military * 2
                        }
                        // Gene `swiss_playbook`: a founded city's Redoubt is its garrison,
                        // worth what the garrison floor pays a missing troop.
                        BuildingKind::Walls
                            if self.decisions.walls_garrison_value > 0
                                && self.walls_garrison_ratio(c) >= 100 =>
                        {
                            GARRISON_FLOOR_VALUE * self.walls_garrison_ratio(c).min(150) / 150
                                + k.military * 2
                        }
                        BuildingKind::Walls if self.redoubt_garrison() && !c.is_capital => {
                            GARRISON_FLOOR_VALUE + k.military * 2
                        }
                        BuildingKind::Walls => {
                            let border =
                                ctx.enemy_cities.iter().any(|e| e.pos.distance(c.pos) <= 8);
                            if self.own_heirs().any(|h| h.pos == c.pos)
                                || self.hall_guard_city(c)
                                || self.diwan_guard_city(c)
                                || self.wonder_guard_city(c)
                                || self.own_original_capital(c)
                            {
                                // The Heir shelters behind walls: his holder scores the goal.
                                // So does the Hall's city, whose land every empire may cross.
                                900 + k.military * 2
                            } else if threatened
                                || self.assessed_border_threat(c.pos)
                                || self.threat_exposure(c) >= 2
                            {
                                800 + k.military * 2
                            } else if self.aggressive_neighbour(c) {
                                // Only walled cities fire back: the rush counter.
                                300 + k.military * 2
                            } else if border {
                                100 + k.military * 2
                            } else {
                                20
                            }
                        }
                        BuildingKind::GreatLibrary
                        | BuildingKind::Pyramids
                        | BuildingKind::HagiaSophia
                        | BuildingKind::EiffelTower
                        | BuildingKind::IterFusionReactor
                        | BuildingKind::RoyalArsenal
                        | BuildingKind::GrandBazaar => {
                            if !t.relay_started
                                && !threatened
                                && self.missing_great_work() > 0
                                && self
                                    .work_plan
                                    .as_ref()
                                    .is_some_and(|p| p.host == c.id && p.building == building)
                            {
                                900 + k.science * 4
                            } else {
                                return 0;
                            }
                        }
                    }
            }
        }
    }

    /// Roads give a campaign direct value when this city can link the chosen target.
    /// Warlike factions invest more in that mobility even before a target is committed.
    fn roadworks_value(&self, c: &CityView) -> i32 {
        let near = |p: legatum_core::Hex| p != c.pos && p.distance(c.pos) <= data::ROADWORKS_RANGE;
        let enemies = self.ctx.enemy_cities.iter().filter(|e| near(e.pos)).count() as i32;
        let others = self
            .view()
            .cities
            .iter()
            .filter(|o| o.id != c.id && near(o.pos))
            .count() as i32;
        // Sparta's non-war buildings cost 50% more, so it gets more headroom to make this
        // logistics investment compete with its production premium.
        let warlike_value = self.decisions.roadworks_war_base
            + self.decisions.roadworks_war_per_enemy * enemies.min(4);
        let campaign_target_in_range = self.campaigns.active.as_ref().is_some_and(|campaign| {
            self.ctx
                .enemy_cities
                .iter()
                .any(|target| target.id == campaign.target && near(target.pos))
        });
        30 + 10 * others.min(6)
            + self.knobs.military
            + warlike_value
            + if campaign_target_in_range { 300 } else { 0 }
    }

    /// An improvement is worth the yield it adds over the planning horizon against its cost,
    /// priced like the tile score (Food 4, Production 3, Gold 2, Science 3) and scaled by the
    /// empire's economic bent. The automatic upgrade counts for half, since it waits on research.
    /// It competes with Settlers and units in `choose_item`'s time-discounted ranking.
    fn improvement_value(&self, opt: &ProductionOption) -> i32 {
        let Some(preview) = &opt.improvement else {
            return 0;
        };
        let points =
            |y: legatum_core::Yields| 4 * y.food + 3 * y.production + 2 * y.gold + 3 * y.science;
        // A Mine on Iron or Coal adds supply: the Forge or Factory it would put to work.
        let supply = if legatum_core::resources::is_strategic(preview.resource) {
            3 * crate::supply_value::own_gain(
                self.view(),
                preview.resource,
                legatum_core::resources::mine_supply_gain(
                    self.faction,
                    self.view().rule_experiments.faction_rules_off,
                ),
            ) * self.decisions.supply_value_percent
                / 100
        } else {
            0
        };
        let goal_supply = self.factory_goal_supply_value(preview.resource);
        let gain = points(preview.gain)
            + preview.upgrade.as_ref().map_or(0, |u| points(u.gain) / 2)
            + supply;
        let window = 40;
        if (gain <= 0 && goal_supply == 0)
            || opt.turns < 0
            || opt.turns + if goal_supply > 0 { 10 } else { window } > self.turns_remaining()
        {
            return 0;
        }
        // The Forge or Factory its supply lets us build is worth its priority (the Mine is
        // half the way there), so a Mine on the deposit that caps an industry beats lesser work.
        let unlock = if legatum_core::resources::is_strategic(preview.resource) {
            self.decisions.industry_priority
                * crate::supply_value::own_gain(
                    self.view(),
                    preview.resource,
                    legatum_core::resources::mine_supply_gain(
                        self.faction,
                        self.view().rule_experiments.faction_rules_off,
                    ),
                )
                * self.decisions.supply_value_percent
                / 200
        } else {
            0
        };
        (50 + self.knobs.economy) * gain * 60 / (2 * opt.cost.max(1)) + unlock + goal_supply
    }

    /// What one more `unit` is worth next to the armies we can see, by the class triangle: the
    /// melee line beats mounted, ranged beats melee, mounted beats ranged. A score like any other
    /// production term (not a rule): the more of the enemy force a class counters, and the less of
    /// it counters that class, the more it is worth, so the mix follows what we face. With no
    /// enemy army in sight the value only keeps our own mix balanced (about half melee, a quarter
    /// each ranged and mounted), so no class is skipped for ever.
    pub(crate) fn composition_bonus(&self, unit: UnitKind) -> i32 {
        use legatum_core::UnitClass::{Melee, Mounted, Ranged};
        let Some(class) =
            composition::class(unit).filter(|c| matches!(c, Melee | Ranged | Mounted))
        else {
            return 0;
        };
        let v = self.view();
        let slot = |c: legatum_core::UnitClass| match c {
            Melee => Some(0),
            Ranged => Some(1),
            Mounted => Some(2),
            _ => None,
        };
        let mut seen = [0i32; 3];
        let note = |kind: UnitKind, seen: &mut [i32; 3]| {
            if let Some(i) = composition::class(kind).and_then(slot) {
                seen[i] += 1;
            }
        };
        for e in &self.ctx.enemy_units {
            note(e.kind, &mut seen);
        }
        for o in &self.ctx.remembered_threats {
            note(o.kind, &mut seen);
        }
        // Neighbours at peace are the army a war would meet.
        for u in &v.units {
            if u.military
                && !u.owner.is_barbarian()
                && !v.same_team(v.me, u.owner)
                && !v.at_war(v.me, u.owner)
                && self
                    .ctx
                    .own_cities
                    .iter()
                    .any(|c| c.pos.distance(u.pos) <= 8)
            {
                note(u.kind, &mut seen);
            }
        }
        let faced: i32 = seen.iter().sum();
        // A mounted unit with a bow (the Keshig) still rides down ranged units, and its arrows
        // answer a melee line as a ranged unit's do (no spear bonus against a shot), which
        // cancels the melee line's counter to it.
        let bow = composition::dual(
            unit,
            self.view()
                .rule_experiments
                .unit_stats_for(self.faction, unit)
                .ranged,
        );
        let counter = |s: &[i32; 3]| match class {
            Melee => s[2] - s[1],
            Ranged => s[0] - s[2],
            _ if bow => s[1],
            // A horse can ride away from a distant spear, while a stationary horse is
            // still vulnerable. A spear beside it after a charge gets the full answer.
            _ if legatum_core::data::unit_era_tier(unit)
                < legatum_core::combat::TRIANGLE_END_ERA_TIER =>
            {
                s[1] - s[0] / 2
            }
            _ => s[1] - s[0],
        };
        // A class is worth more the more of the enemy army it counters; one that is itself
        // countered is worth nothing extra (never a penalty against buildings and Settlers).
        let matchup = if faced >= 2 {
            (MATCHUP_VALUE * counter(&seen) / faced).max(0)
        } else {
            0
        };
        let mut own = [0i32; 3];
        for u in &self.ctx.own_units {
            note(u.kind, &mut own);
        }
        let own_total: i32 = own.iter().sum();
        let target = [5, 3, 2];
        let me = slot(class).unwrap_or(0);
        let balance = (BALANCE_VALUE * (target[me] * own_total - 10 * own[me])
            / (10 * (own_total + 1)))
            .clamp(0, BALANCE_VALUE);
        let fast = if class == Mounted && self.strategy == Some(crate::Strategy::Rush) {
            RUSH_MOUNTED_VALUE
        } else {
            0
        };
        matchup + balance + fast + self.walled_quota_value(class, &own)
    }

    /// Gene `walled_quota`: the war our plan fights marches on a walled city, so shooters are
    /// worth as much as countering the army we face until the gene's count stand in our army,
    /// plus a melee escort of two to take the city once its garrison is shot dead (playtest
    /// 8.9.2: the Mongols took walled Aarau with four or five shooters a tier ahead of the walls
    /// and the melee held back; every other AI at war in that position took nothing).
    fn walled_quota_value(&self, class: legatum_core::UnitClass, own: &[i32; 3]) -> i32 {
        use legatum_core::UnitClass::{Melee, Ranged};
        let quota = self.decisions.walled_quota;
        if quota <= 0 || !self.hard_defense() {
            return 0;
        }
        let v = self.view();
        let Some(t) = self.routed_plan().and_then(|p| p.kind.war_target()) else {
            return 0;
        };
        if !v.teams_at_war(v.team, t) {
            return 0;
        }
        let walled = self
            .ctx
            .enemy_cities
            .iter()
            .filter(|c| v.team_of(c.owner) == t)
            .min_by_key(|c| {
                self.ctx
                    .own_units
                    .iter()
                    .filter(|u| u.military)
                    .map(|u| u.pos.distance(c.pos))
                    .min()
                    .unwrap_or(99)
            })
            .is_some_and(|c| c.walls);
        if !walled {
            return 0;
        }
        match class {
            Ranged if own[1] < quota => MATCHUP_VALUE,
            Melee if own[0] < 2 => BALANCE_VALUE,
            _ => 0,
        }
    }

    fn unique_unit_bonus(&self, unit: UnitKind) -> i32 {
        let d = &self.decisions;
        let class = |is: bool, bonus: i32| if is { bonus } else { 0 };
        class(data::is_veteran_infantry(unit), d.veteran_infantry_bonus)
            + class(data::is_infantry(unit), d.infantry_bonus)
            + class(self.mongol_horde() && unit == UnitKind::Cataphract, 100)
    }

    pub(crate) fn mongol_horde(&self) -> bool {
        self.faction == Faction::Mongols
            && self.difficulty == legatum_core::AiDifficulty::Hard
            && self.decisions.mongol_horde > 0
    }

    fn best_mounted_option(&self, city: &CityView) -> Option<UnitKind> {
        composition::MOUNTED_LADDER.into_iter().find(|kind| {
            city.detail.as_ref().is_some_and(|d| {
                d.options
                    .iter()
                    .any(|o| o.item == ProductionItem::Unit { unit: *kind })
            })
        })
    }

    fn goal_building_bonus(&self, c: &CityView, b: BuildingKind, turns: i32) -> i32 {
        let done = self
            .view()
            .players
            .iter()
            .find(|p| p.id == self.ctx.me)
            .is_some_and(|p| p.faction_goal_done);
        (match (self.faction, b) {
            (Faction::Germany, _) => self.factory_goal_stage_value(c, b, turns),
            (Faction::Athens, BuildingKind::Library) => {
                // Completing the third lab is a concrete goal investment, not another
                // interchangeable science building. The earlier labs keep their usual value.
                let labs = self
                    .ctx
                    .own_cities
                    .iter()
                    .filter(|city| {
                        city.detail.as_ref().is_some_and(|d| {
                            d.resistance <= 0
                                && legatum_core::data::has_building(
                                    &d.buildings,
                                    BuildingKind::Library,
                                )
                        })
                    })
                    .count();
                if !done
                    && labs == 2
                    && (self
                        .view()
                        .research
                        .known
                        .contains(&legatum_core::TechId::Philosophy)
                        || self.view().research.current == Some(legatum_core::TechId::Philosophy))
                {
                    1800
                } else {
                    300
                }
            }
            // The lab network and Relay need production behind their science investment.
            (Faction::Athens, BuildingKind::Workshop) => 500,
            _ => 0,
        }) + if b == BuildingKind::Walls
            && self
                .ctx
                .enemy_cities
                .iter()
                .any(|e| e.pos.distance(c.pos) <= 8)
        {
            self.decisions.walls_near_enemy_bonus
        } else {
            0
        }
    }

    /// Current rules have permanent war. Count successful purchases in the owner view,
    /// not brain memory, so loading a save never resets this budget.
    pub(crate) fn military_purchase_available(&self) -> bool {
        // Permanent war can span the whole game and several cities. Bound optional
        // reinforcements and title investments so the opening cannot spend everything,
        // but renew the allowance with time: a lifetime cap left later treasuries idle.
        // One more purchase per 50 Standard turns.
        let base = if self.knobs.military >= 70 { 6 } else { 4 };
        let cap = base + self.view().turn / 50;
        let pending = self
            .orders
            .iter()
            .filter(|o| {
                matches!(o,
            Order::BuyItem { item: ProductionItem::Unit { unit }, .. }
                if self.view().rule_experiments.unit_stats_for(self.faction, *unit).military)
            })
            .count() as u32;
        self.view()
            .economy
            .military_purchases
            .saturating_add(pending)
            < cap
    }

    /// The threat pass's exposure becomes a production concern when a rival
    /// army is also visible close enough to arrive before a long build finishes.
    fn visible_threat_approach(&self, c: &CityView) -> bool {
        self.difficulty == AiDifficulty::Hard
            && self.decisions.production_threat_agreement > 0
            && self.threat_exposure(c) > 0
            && self.view().units.iter().any(|u| {
                u.military
                    && !u.owner.is_barbarian()
                    && !self.view().same_team(self.view().me, u.owner)
                    && u.pos.distance(c.pos) <= 4
            })
    }

    /// The shooter need used by both production passes. The first value is an
    /// approaching bombardment; the second covers a nearby hostile at a
    /// garrisoned city. Under the agreement gene, it also reads the threat
    /// pass's exposure when a visible rival army approaches within four hexes.
    fn ranged_relief_needed(&self, c: &CityView) -> (bool, bool) {
        let bombardment = self.ctx.threat_units.iter().any(|u| {
            u.ranged > 0
                && u.pos.distance(c.pos) <= u.range + 1
                && (self.opening.is_none() || !u.owner.is_barbarian())
        });
        let shooter_here = self
            .ctx
            .own_units
            .iter()
            .any(|u| u.ranged > 0 && u.pos.distance(c.pos) <= 1);
        let garrison_here = self
            .ctx
            .own_units
            .iter()
            .any(|u| u.military && u.pos == c.pos);
        let exposed_approach = self.visible_threat_approach(c);
        let needs_ranged = (bombardment
            && !self.ctx.own_units.iter().any(|u| {
                u.ranged > 0
                    && self.ctx.threat_units.iter().any(|e| {
                        e.ranged > 0
                            && e.pos.distance(c.pos) <= e.range + 1
                            && u.pos.distance(e.pos) <= u.range
                    })
            }))
            || (!shooter_here
                && ((self
                    .ctx
                    .threat_units
                    .iter()
                    .any(|u| u.military && u.pos.distance(c.pos) <= 3)
                    && garrison_here)
                    || (exposed_approach && (garrison_here || c.walls))));
        (bombardment, needs_ranged)
    }

    fn plan_emergency_production(&mut self) {
        let military = self.upkeep_unit_count();
        let purchased = self.planned_military_purchases();
        let upkeep = |units: i32| {
            let cost = legatum_core::data::paid_military_upkeep_rules(
                self.view().rule_experiments.faction_rules_off,
                self.faction,
                (units - legatum_core::data::FREE_MILITARY_UNITS).max(0),
            );
            if self
                .view()
                .research
                .known
                .contains(&legatum_core::TechId::MilitaryEngineering)
            {
                (cost * 3 + 3) / 4
            } else {
                cost
            }
        };
        let added_upkeep = upkeep(military + purchased + 1) - upkeep(military);
        for (ci, c) in self.ctx.own_cities.iter().enumerate() {
            let d = c.detail.as_ref().expect("own city");
            let exposed_capture = self.last_city_front(c)
                || self.ctx.own_cities.iter().any(|front| {
                    front.id != c.id
                        && front.hp * 2 <= front.max_hp
                        && c.pos.distance(front.pos) <= 8
                        // Resistance alone is not an army threatening neighboring cities.
                        && ((front.detail.as_ref().is_some_and(|d| d.resistance > 0)
                            && !(self.hard_defense() && self.decisions.capture_alarm > 0))
                            || self.ctx.enemy_units.iter().any(|enemy| {
                                enemy.military
                                    && !enemy.owner.is_barbarian()
                                    && enemy.pos.distance(front.pos) <= 6
                            }))
                });
            if self.primitive_barbarian_guard(ci)
                && d.producing
                    == Some(ProductionItem::Unit {
                        unit: UnitKind::Settler,
                    })
                && !exposed_capture
            {
                continue;
            }
            if d.resistance > 0
                || self.orders.iter().any(|o| {
                    matches!(o,
                Order::BuyItem { city, item } if *city == c.id && is_military_unit(*item))
                })
            {
                continue;
            }
            let (bombardment, needs_ranged) = self.ranged_relief_needed(c);
            let shooter_here = self
                .ctx
                .own_units
                .iter()
                .any(|u| u.ranged > 0 && u.pos.distance(c.pos) <= 1);
            let threat = self.ctx.threat[ci]
                - if self.opening.is_some() {
                    self.ctx.barbarian_threat[ci]
                } else {
                    0
                };
            if !needs_ranged && threat <= self.local_defense(c) && !exposed_capture {
                continue;
            }
            // The routine pass now prices this same shooter need. Leave a
            // shooter-only choice to its scored comparison with work in progress.
            if self.difficulty == AiDifficulty::Hard
                && self.decisions.production_threat_agreement > 0
                && threat <= self.local_defense(c)
                && !exposed_capture
            {
                continue;
            }
            // A shooter wanted only by the standing preference (an enemy unit at three hexes,
            // no bombardment) is the routine pass's to weigh against what is under way; the
            // emergency swap was undone by it the next turn (gene `emergency_bombardment_only`).
            if self.decisions.emergency_bombardment_only > 0
                && !bombardment
                && !exposed_capture
                && threat <= self.local_defense(c)
            {
                continue;
            }
            // Walls under way are the city's durable answer: finish them with Production,
            // never swap them for a unit (gene `keep_walls`; playtest
            // 8.4: a Byzantium capital swapped Walls at 86 of 91 Production for an Archer it
            // never finished, and fell three turns later).
            if self.hard_defense()
                && self.decisions.keep_walls > 0
                && let Some(ProductionItem::Building { building }) = d.producing
                && data::building_family(building) == BuildingKind::Walls
                && (self.decisions.siege_survival_clock == 0
                    || self
                        .defense_risk(c, None)
                        .production_deadline()
                        .is_none_or(|fall| d.turns_to_complete.is_some_and(|turns| turns < fall)))
            {
                continue;
            }
            // Preserve an adequate defender already under construction, and a city
            // investment finishing now unless the city is forecast to fall this turn.
            let falls = self.defense_risk(c, None).falls;
            if self.decisions.air_commitment > 0
                && !falls
                && d.producing.is_some_and(|item| {
                    matches!(item,
                    ProductionItem::Unit { unit } if crate::air::useful_build(self.view(), c, unit))
                })
            {
                continue;
            }
            // A Settler well under way is finished, not swapped: it cost a citizen and most of
            // its Production, and a swapped one waits for the next quiet turn (gene
            // `keep_settler`; playtest 8.4 swapped one at 80 of 163 for a Spearman). Only a city
            // that is forecast to fall, or has a hostile armed unit within two hexes, gives it up.
            if self.keep_committed_settler(c) {
                continue;
            }
            // A screened shooter gives an unwalled city time to build Walls. Do not
            // replace that durable answer with a second melee unit just because a
            // rival has been sighted at the border.
            if !falls
                && shooter_here
                && !bombardment
                && !c.walls
                && d.options.iter().any(|o| {
                    o.item
                        == (ProductionItem::Building {
                            building: BuildingKind::Walls,
                        })
                })
            {
                continue;
            }
            let fall_turn = if self.hard_defense() && self.decisions.siege_survival_clock > 0 {
                self.defense_risk(c, None).production_deadline()
            } else {
                None
            };
            if d.producing.is_some_and(|item| {
                is_military_unit(item)
                    && (!needs_ranged
                        || matches!(item, ProductionItem::Unit { unit }
                    if self.view().rule_experiments.unit_stats_for(self.faction, unit).ranged > 0))
            }) && fall_turn
                .is_none_or(|fall| d.turns_to_complete.is_some_and(|turns| turns < fall))
                || (!falls && d.turns_to_complete == Some(1))
            {
                continue;
            }
            let best = d
                .options
                .iter()
                .filter(|o| {
                    is_military_unit(o.item)
                        && o.turns > 0
                        && fall_turn.is_none_or(|fall| o.turns < fall)
                })
                .filter(|o| {
                    !needs_ranged
                        || matches!(o.item, ProductionItem::Unit { unit }
                    if self.view().rule_experiments.unit_stats_for(self.faction, unit).ranged > 0)
                })
                .filter(|o| {
                    // Do not interrupt the route out of bankruptcy for a unit that will
                    // disband on completion. Include the quoted tile-focus change, the
                    // free allowance and the owner's researched upkeep discount.
                    if exposed_capture
                        && d.producing
                            == Some(ProductionItem::Unit {
                                unit: UnitKind::Settler,
                            })
                    {
                        // The current civilian costs population and cannot defend the
                        // captured front. Even an upkeep-limited replacement troop has
                        // more immediate value than finishing that civilian.
                        return true;
                    }
                    let income = self.view().economy.gold_per_turn.total + o.yields.total.gold
                        - d.yields.total.gold;
                    let reserve = (added_upkeep - income).max(0) * 10;
                    self.gold_left + income * o.turns >= reserve
                })
                .max_by_key(|o| {
                    let ProductionItem::Unit { unit } = o.item else {
                        unreachable!()
                    };
                    let stats = self
                        .view()
                        .rule_experiments
                        .unit_stats_for(self.faction, unit);
                    (
                        (self.matchup_strength(unit, c.pos)
                            + if needs_ranged && stats.ranged > 0 {
                                20
                            } else {
                                0
                            })
                            * fall_turn.map_or(1, |fall| (fall - o.turns).max(1))
                            * 1000
                            / (o.turns + 2),
                        std::cmp::Reverse(o.cost),
                        std::cmp::Reverse(o.item),
                    )
                });
            if let Some(opt) = best {
                self.orders.push(Order::SetProduction {
                    city: c.id,
                    item: opt.item,
                });
            } else if exposed_capture
                && d.producing
                    == Some(ProductionItem::Unit {
                        unit: UnitKind::Settler,
                    })
                && let Some(opt) = d
                    .options
                    .iter()
                    .filter(|o| {
                        !matches!(
                            o.item,
                            ProductionItem::Unit {
                                unit: UnitKind::Settler
                            }
                        ) && (matches!(
                            o.item,
                            ProductionItem::Building {
                                building: BuildingKind::Market
                            }
                        ) || o.yields.total.gold > d.yields.total.gold)
                    })
                    .min_by_key(|o| (o.turns, o.cost))
            {
                // When another troop would immediately disband, rebuild income instead
                // of finishing a civilian while the new city is exposed.
                self.orders.push(Order::SetProduction {
                    city: c.id,
                    item: opt.item,
                });
            }
        }
    }

    /// Gold a held, live treasury goal (the Swiss hoard) keeps when its points are what
    /// hold the team at its VP target. Readiness and frontline purchases stay above it
    /// (goals-3 stall: a Swiss at 10 VP bought two defenders the turn after reaching its
    /// hoard, fell to 8, and flickered between 10 and 8 for 1,000 turns); a city about to fall
    /// (`buy_emergency_defender`) still may spend it.
    pub(crate) fn live_goal_floor(&self) -> i32 {
        if self.decisions.endgame_push == 0 || !self.hoard_held() {
            return 0;
        }
        let v = self.view();
        let at_target = v
            .standings
            .iter()
            .find(|s| s.team == v.team)
            .is_some_and(|s| s.vp + 2 >= s.target);
        if !at_target {
            return 0;
        }
        legatum_core::goals::SWISS_HOARD_GOLD + (-v.economy.gold_per_turn.total).max(0)
    }

    fn buy_emergency_defender(&mut self) -> bool {
        if let Some(fund) = self.named_fund()
            && fund.kind == crate::gold_hold::FundKind::Emergency
        {
            if self
                .ctx
                .own_cities
                .iter()
                .copied()
                .find(|c| Some(c.id) == fund.city)
                .is_some_and(|c| self.military_upgrade_hold(c))
            {
                return false;
            }
            let keep = (self.marginal_military_upkeep() - self.view().economy.gold_per_turn.total)
                .max(0)
                .max(self.live_goal_floor());
            if fund.ready {
                let cancelled: Vec<_> = self
                    .orders
                    .iter()
                    .filter_map(|order| match order {
                        Order::UpgradeSiege { unit } => Some(*unit),
                        _ => None,
                    })
                    .collect();
                let refund = cancelled
                    .iter()
                    .filter_map(|id| {
                        self.view()
                            .units
                            .iter()
                            .find(|u| u.id == *id)
                            .and_then(|u| u.siege_offer.as_ref())
                            .map(|offer| offer.price.total)
                    })
                    .sum::<i32>();
                if !cancelled.is_empty() && self.gold_left + refund - fund.cost >= keep {
                    self.gold_left += refund;
                    self.orders
                        .retain(|order| !matches!(order, Order::UpgradeSiege { .. }));
                }
            }
            if self.decision_trace.is_some() {
                self.note_option(crate::DecisionOption::new(
                    "plan",
                    format!(
                        "emergency release: Gold {}, fund {}, floor {}, prior buy {}",
                        self.gold_left,
                        fund.cost,
                        keep,
                        self.orders
                            .iter()
                            .any(|o| matches!(o, Order::BuyItem { .. }))
                    ),
                    "audit",
                ));
            }
            if !fund.ready
                || self.gold_left - fund.cost < keep
                || self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::BuyItem { .. }))
            {
                return false;
            }
            let Some(front) = self.ctx.own_cities.iter().find(|c| Some(c.id) == fund.city) else {
                return false;
            };
            let front_pos = front.pos;
            let need = (2 - self
                .ctx
                .own_units
                .iter()
                .filter(|u| {
                    u.military && u.kind != UnitKind::Scout && u.pos.distance(front_pos) <= 2
                })
                .count() as i32)
                .max(0);
            let mut cities = self.ctx.own_cities.clone();
            cities.sort_by_key(|c| (c.pos.distance(front_pos), c.id));
            let mut bought = 0;
            let mut budget = fund.cost;
            for c in cities {
                if bought >= need || c.pos.distance(front_pos) > 4 {
                    continue;
                }
                let Some(d) = c.detail.as_ref() else {
                    continue;
                };
                let best = d
                    .options
                    .iter()
                    .filter_map(|o| {
                        let ProductionItem::Unit { unit } = o.item else {
                            return None;
                        };
                        if !is_military_unit(o.item)
                            || unit == UnitKind::Scout
                            || o.buy_block.is_some()
                        {
                            return None;
                        }
                        let cost = o.buy_cost.filter(|cost| {
                            *cost > 0 && *cost <= budget && self.gold_left - *cost >= keep
                        })?;
                        let strength = self
                            .view()
                            .rule_experiments
                            .unit_stats_for(self.faction, unit)
                            .strength;
                        let survives = self
                            .view()
                            .units
                            .iter()
                            .filter(|u| {
                                u.military
                                    && !u.owner.is_barbarian()
                                    && self.view().at_war(self.view().me, u.owner)
                                    && u.pos.distance(front_pos) <= 8
                            })
                            .all(|u| {
                                legatum_core::combat::base_damage(u.strength, strength)
                                    < data::UNIT_HP
                            });
                        let rank = if self.decisions.hard_purchase_scorer > 0 {
                            self.military_purchase_score(unit, cost, c.pos)
                        } else {
                            i32::from(data::unit_era_tier(unit))
                        };
                        survives.then_some((rank, strength, -cost, cost, o.item))
                    })
                    .max_by_key(|(rank, strength, neg_cost, _, _)| (*rank, *strength, *neg_cost));
                if let Some((_, _, _, cost, item)) = best {
                    self.gold_left -= cost;
                    budget -= cost;
                    self.orders.push(Order::BuyItem { city: c.id, item });
                    bought += 1;
                }
            }
            if bought > 0 {
                self.tag_buys("named_emergency");
            }
            return false;
        }
        let mut cities = self.ctx.own_cities.clone();
        cities.sort_by_key(|c| (c.hp, c.id));
        for c in cities {
            let d = c.detail.as_ref().expect("own city");
            if d.resistance > 0
                || self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == c.id))
            {
                continue;
            }
            let risk = self.defense_risk(c, None);
            let bombardment = self
                .ctx
                .threat_units
                .iter()
                .any(|u| u.ranged > 0 && u.pos.distance(c.pos) <= u.range);
            if !risk.threatened && !bombardment {
                continue;
            }
            if !risk.falls && self.military_upgrade_hold(c) {
                continue;
            }
            // Save a concrete winning cash goal through sustainable harassment, but
            // never let a lifetime purchase allowance veto saving a falling city.
            if self.saving_for_goal() && !risk.falls {
                continue;
            }
            let occupied = self
                .view()
                .units
                .iter()
                .any(|u| u.military && u.pos == c.pos);
            // A garrison cannot spare a city with HP its damage (Civ 6); relief is a unit
            // that can arrive, survive, and extend the city's survival clock.
            let relief = self.ctx.own_units.iter().any(|u| {
                u.military && self.arrives(u, c.pos) && {
                    let after = self.defense_risk(c, Some((*u).clone()));
                    (after.holds_longer_than(risk) || !after.falls) && !after.garrison_lost
                }
            });
            if !occupied && !relief {
                let upkeep = self.marginal_military_upkeep();
                let keep = (upkeep - self.view().economy.gold_per_turn.total).max(0);
                let mut choices: Vec<_> = d
                    .options
                    .iter()
                    .filter_map(|o| {
                        let ProductionItem::Unit { unit } = o.item else {
                            return None;
                        };
                        if !(composition::is_melee_line(unit)
                            || composition::is_ranged_line(unit)
                            || composition::is_mounted(unit))
                            || o.buy_block.is_some()
                        {
                            return None;
                        }
                        let cost = o.buy_cost?;
                        (cost > 0 && self.gold_left - cost >= keep).then_some((cost, unit, o.item))
                    })
                    .collect();
                choices.sort();
                // The city absorbs attacks while it has HP (Civ 6): a garrison cannot spare
                // it damage, but it must be beaten before the city is captured, and a ranged
                // one shoots every turn in safety. Buy one that survives the projected
                // attacks (or at least saves a falling city), preferring a shooter.
                let mut best = None;
                for (cost, unit, item) in choices {
                    let after =
                        self.defense_risk(c, Some(purchased_defender(self.view(), c, unit)));
                    if (after.falls && !after.holds_longer_than(risk))
                        || (after.garrison_lost && !risk.falls)
                    {
                        continue;
                    }
                    // A shooter only while the city is healthy: a badly damaged city may
                    // fall soon, and then its garrison must survive melee.
                    // A unit with both (the Keshig) is the right role either way.
                    let ranged = self
                        .view()
                        .rule_experiments
                        .unit_stats_for(self.faction, unit)
                        .ranged;
                    let legacy_key = (
                        if self.decisions.named_fund > 0 && self.difficulty == AiDifficulty::Hard {
                            -i32::from(data::unit_era_tier(unit))
                        } else {
                            0
                        },
                        after.garrison_lost,
                        !composition::dual(unit, ranged)
                            && (ranged == 0) == (c.hp * 2 > data::CITY_HP),
                        cost,
                    );
                    let score = self.military_purchase_score(unit, cost, c.pos);
                    let key = if self.decisions.hard_purchase_scorer > 0
                        && self.difficulty == AiDifficulty::Hard
                    {
                        (0, after.garrison_lost, false, -score)
                    } else {
                        legacy_key
                    };
                    if best.is_none_or(|(k, _, _)| key < k) {
                        best = Some((key, cost, item));
                    }
                }
                if let Some((_, cost, item)) = best {
                    self.orders.push(Order::BuyItem { city: c.id, item });
                    self.gold_left -= cost;
                    return true;
                }
            }
            // A melee garrison cannot answer repeated bombardment of its city. Buy
            // ranged relief into an engine-quoted free spawn tile when local shooters
            // cannot match the visible pressure. Defense uses an upkeep budget, not
            // the lifetime allowance for optional Army-title purchases.
            let ranged_pressure: i32 = self
                .ctx
                .threat_units
                .iter()
                .filter(|u| u.ranged > 0 && u.pos.distance(c.pos) <= u.range + 1)
                .map(|u| u.ranged * u.hp / 100)
                .sum();
            let ranged_cover: i32 = self
                .ctx
                .own_units
                .iter()
                .filter(|u| {
                    u.ranged > 0
                        && self.ctx.threat_units.iter().any(|e| {
                            e.ranged > 0
                                && e.pos.distance(c.pos) <= e.range + 1
                                && u.pos.distance(e.pos) <= u.range
                        })
                })
                .map(|u| u.ranged * u.hp / 100)
                .sum();
            if ranged_pressure > ranged_cover {
                let keep = (self.marginal_military_upkeep()
                    - self.view().economy.gold_per_turn.total)
                    .max(0)
                    * 10;
                let keep = keep.max(self.live_goal_floor());
                let shooter = d
                    .options
                    .iter()
                    .filter_map(|o| {
                        let ProductionItem::Unit { unit } = o.item else {
                            return None;
                        };
                        if !composition::is_ranged_line(unit) {
                            return None;
                        }
                        let cost = o.buy_cost?;
                        (o.buy_block.is_none() && cost > 0 && self.gold_left - cost >= keep)
                            .then_some((
                                if self.decisions.hard_purchase_scorer == 0
                                    && self.decisions.named_fund > 0
                                    && self.difficulty == AiDifficulty::Hard
                                {
                                    i32::from(data::unit_era_tier(unit))
                                } else {
                                    0
                                },
                                self.military_purchase_score(unit, cost, c.pos),
                                std::cmp::Reverse(cost),
                                o.item,
                            ))
                    })
                    .max();
                if let Some((_, _, std::cmp::Reverse(cost), item)) = shooter {
                    self.orders.push(Order::BuyItem { city: c.id, item });
                    self.gold_left -= cost;
                    return true;
                }
            }
        }
        false
    }

    fn buy_ready_defenders(&mut self) {
        if self.named_fund().is_some() {
            return;
        }
        // A forecast rescue has already had priority. Optional readiness must
        // wait until the Swiss treasury award is actually banked.
        if self.faction == Faction::Switzerland && self.goal_pending() {
            return;
        }
        let mut cities: Vec<_> = self
            .ctx
            .own_cities
            .iter()
            .enumerate()
            .map(|(i, c)| (self.ctx.threat[i] - self.ctx.barbarian_threat[i], *c))
            .collect();
        cities
            .sort_by_key(|(threat, c)| (std::cmp::Reverse(*threat - self.local_defense(c)), c.id));
        for (threat, c) in cities {
            if threat == 0
                || self.local_defense(c) >= threat * 5 / 4
                || self
                    .orders
                    .iter()
                    .any(|o| matches!(o,Order::BuyItem {city,..} if *city==c.id))
            {
                continue;
            }
            let d = c.detail.as_ref().expect("own city");
            if d.resistance > 0 {
                continue;
            }
            if self.military_upgrade_hold(c) {
                continue;
            }
            let units = self.upkeep_unit_count();
            let planned = self.planned_military_purchases();
            let planned_upkeep =
                self.military_upkeep_at(units + planned) - self.military_upkeep_at(units);
            let keep = (self.marginal_military_upkeep() + planned_upkeep
                - self.view().economy.gold_per_turn.total)
                .max(0)
                * 10;
            let keep = keep.max(self.live_goal_floor());
            // Gene `buy_hold` 2: readiness leaves the held Gold unless an attack can arrive now.
            let keep = if self.decisions.buy_hold >= 2 && !self.defense_risk(c, None).threatened {
                keep.max(self.gold_reserve() + self.held_gold())
            } else {
                keep
            };
            let melee = self.ctx.own_units.iter().any(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && composition::charges(u.kind)
                    && u.pos.distance(c.pos) <= 1
            });
            let shooter = self
                .ctx
                .own_units
                .iter()
                .any(|u| u.ranged > 0 && u.pos.distance(c.pos) <= 2);
            // The visible attackers determine the marginal value of a class counter.
            // Keep the ordinary strength per Gold and role scores as tie breakers.
            let counter_value = |unit| {
                self.ctx
                    .threat_units
                    .iter()
                    .filter(|e| e.military && !e.owner.is_barbarian() && e.pos.distance(c.pos) <= 3)
                    .map(|e| {
                        let bonus = legatum_core::combat::class_bonus_for_rules(
                            self.view().rule_experiments.faction_rules_off,
                            self.faction,
                            unit,
                            e.kind,
                        )
                        .1;
                        bonus * e.hp * (4 - e.pos.distance(c.pos)) / 100
                    })
                    .sum::<i32>()
            };
            let best = d
                .options
                .iter()
                .filter_map(|o| {
                    let ProductionItem::Unit { unit } = o.item else {
                        return None;
                    };
                    if !is_military_unit(o.item) || o.buy_block.is_some() {
                        return None;
                    }
                    let cost = o
                        .buy_cost
                        .filter(|cost| *cost > 0 && self.gold_left - *cost >= keep)?;
                    let stats = self
                        .view()
                        .rule_experiments
                        .unit_stats_for(self.faction, unit);
                    let role = if !melee {
                        composition::charges(unit)
                    } else {
                        !shooter && stats.ranged > 0
                    };
                    let score = self.military_purchase_score(unit, cost, c.pos);
                    let counter = counter_value(unit);
                    let (first, second, third, fourth) = if self.decisions.hard_purchase_scorer > 0
                        && self.difficulty == AiDifficulty::Hard
                    {
                        (0, score, role, counter)
                    } else {
                        (
                            if self.decisions.named_fund > 0
                                && self.difficulty == AiDifficulty::Hard
                            {
                                i32::from(data::unit_era_tier(unit))
                            } else {
                                0
                            },
                            counter,
                            role,
                            score,
                        )
                    };
                    Some((
                        first,
                        second,
                        third,
                        fourth,
                        std::cmp::Reverse(cost),
                        o.item,
                    ))
                })
                .max();
            if let Some((_, _, _, _, std::cmp::Reverse(cost), item)) = best {
                self.orders.push(Order::BuyItem { city: c.id, item });
                self.gold_left -= cost;
            }
        }
    }

    fn plan_buy(&mut self, tally: &mut Tally) {
        let named = self.named_fund();
        if named.is_none()
            && self.campaigns.active.is_some()
            && self.view().economy.gold_per_turn.total < 0
        {
            return;
        }
        if named.is_none() && self.saving_wins() {
            return;
        }
        if named.is_some_and(|fund| fund.kind == crate::gold_hold::FundKind::Emergency) {
            return;
        }
        // A funded Great Work can be bought before the treasury award when
        // the purchase still leaves enough Gold to bank that award this turn.
        // Buying one can also finish the public VP race immediately.
        if named.is_none() && self.faction == Faction::Switzerland {
            let goal = legatum_core::goals::SWISS_HOARD_GOLD;
            let work_wins = self.view().standings.iter().any(|standing| {
                standing.team == self.view().team && standing.vp + 2 >= standing.target
            });
            for city in &self.ctx.own_cities {
                let detail = city.detail.as_ref().expect("own city detail");
                if self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::BuyItem { city: id, .. } if *id == city.id))
                {
                    continue;
                }
                if let Some(option) = detail.options.iter().find(|option| {
                    data::is_great_work_item(option.item)
                        && (work_wins || self.work_plan.as_ref().is_some_and(|plan| {
                            plan.host == city.id && matches!(option.item,
                                ProductionItem::Wonder { building, .. } | ProductionItem::Building { building }
                                if building == plan.building)
                        }))
                        && option.buy_block.is_none()
                        && option.buy_cost.is_some_and(|cost| {
                            self.gold_left >= cost
                                && ((!self.goal_pending() && !self.hoard_held())
                                    || work_wins
                                    || self.gold_left - cost
                                        + self.view().economy.gold_per_turn.total.min(0)
                                        >= goal)
                        })
                }) {
                    let cost = option.buy_cost.expect("affordable quote");
                    self.gold_left -= cost;
                    self.orders.push(Order::BuyItem {
                        city: city.id,
                        item: option.item,
                    });
                    self.tag_buys("swiss_work");
                    return;
                }
            }
        }
        // Gene `decisive_buy`: a purchase whose forecast capture is worth more VP than the
        // reserve holds may spend it, whatever the plan is called.
        if let Some(item) = self.buy_decisive() {
            tally.add(item);
            self.tag_buys("decisive");
            return;
        }
        if self.named_fund().is_some() {
            return;
        }
        // Gene `hollow_march`: before the frontline defender, whose danger keeps the whole
        // treasury (the Swiss hoard reserve) for a defender it never buys.
        if self.buy_war_army(tally) {
            self.tag_buys("war_army");
            return;
        }
        // A threatened city's unaffordable defender holds its price back; it doesn't end
        // every other purchase (playtest 8.9.17: the Swiss froze 768 Gold to 1,600 with
        // Mongol Horsemen in sight).
        let hold = match self.buy_frontline_defender(tally) {
            Frontline::Bought => {
                self.tag_buys("frontline");
                return;
            }
            Frontline::Save(gold) => gold,
            Frontline::Clear => 0,
        };
        // Gene `escort_buy`: a second troop before the first Settler meets a visible army.
        let hold = match self.second_troop_buy() {
            Some(Ok((city, item, cost))) => {
                self.gold_left -= cost;
                self.orders.push(Order::BuyItem { city, item });
                tally.add(item);
                self.tag_buys("second_troop");
                return;
            }
            Some(Err(gold)) => hold.max(gold),
            None => hold,
        };
        // Spend awarded treasury on the named Great Work route before a
        // generic upgrade uses the same city slot. A Work is rarely buyable
        // until several supporting buildings have been completed.
        if self.faction == Faction::Switzerland
            && !self.goal_pending()
            && self.view().turn >= self.decisions.work_plan_turn as u32
            && let Some(plan) = &self.work_plan
            && plan.turns <= self.turns_remaining()
        {
            let reserve = self
                .gold_reserve()
                .max(self.frontline_upgrade_reserve())
                .max(hold);
            for &(city, building) in &plan.steps {
                let Some(detail) = self
                    .ctx
                    .own_cities
                    .iter()
                    .find(|c| c.id == city)
                    .and_then(|c| c.detail.as_ref())
                else {
                    continue;
                };
                if self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::BuyItem { city: id, .. } if *id == city))
                {
                    continue;
                }
                if let Some((item, cost)) = detail.options.iter()
                    .filter(|o| matches!(o.item, ProductionItem::Wonder { building: b, .. } if b == building)
                        && o.turns > 1 && o.buy_block.is_none())
                    .filter_map(|o| o.buy_cost.map(|cost| (o, cost)))
                    .filter(|(_, cost)| self.gold_left - *cost >= reserve)
                    .min_by_key(|(o, _)| self.wonder_tile_penalty(o))
                    .map(|(o, cost)| (o.item, cost)) {
                    self.gold_left -= cost;
                    self.orders.push(Order::BuyItem { city, item });
                    self.tag_buys("work_plan");
                    return;
                }
            }
        }
        // Gene `buy_hold`: routine purchases leave the Gold the spike and emergencies want.
        let held = self.held_gold();
        let reserve = self
            .gold_reserve()
            .max(self.frontline_upgrade_reserve())
            .max(hold)
            .max(self.gold_reserve() + held);
        let mut best: Option<(
            i32,
            std::cmp::Reverse<i32>,
            std::cmp::Reverse<legatum_core::CityId>,
            ProductionItem,
        )> = None;
        // An affordable-only search repeatedly empties a treasury on small buys while a
        // better building stays one or two income turns away. Under the savings gene,
        // compare total decision value among near-term purchases; Gold price enters as
        // affordability and arrival time. The default retains value-per-Gold ranking.
        let mut future: Option<(i32, i32, i32, i32)> = None; // value, quote, reserve, ETA
        let net = if self.decisions.purchase_savings_turns > 0 {
            self.planned_net_gold(0)
        } else {
            0
        };
        for (ci, c) in self.ctx.own_cities.iter().enumerate() {
            let d = c.detail.as_ref().expect("own city detail");
            if d.resistance > 0
                || self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == c.id))
            {
                continue;
            }
            for opt in &d.options {
                // Emergency military and scoring purchases have their own forecast gate.
                // Natural production supplies ordinary reinforcements.
                if matches!(opt.item, ProductionItem::Unit { unit } if unit != UnitKind::Settler)
                    || opt.turns <= 1
                    || legatum_core::data::is_great_work_item(opt.item)
                    || opt.buy_block.is_some()
                        && (self.decisions.purchase_savings_turns == 0
                            || !matches!(
                                opt.buy_block,
                                Some(legatum_core::economy::BuyBlock::NotEnoughGold)
                            ))
                {
                    continue;
                }
                // A final Market's savings must not freeze a young colony's growth.
                // Food buildings add population, and a claimed tile per growth; they still pay
                // the normal reserve and any pending frontline upgrade.
                let purchase_reserve = if matches!(
                    opt.item,
                    ProductionItem::Building {
                        building: BuildingKind::Granary | BuildingKind::RiceStorehouse,
                    }
                ) {
                    self.ordinary_gold_reserve()
                        .max(self.frontline_upgrade_reserve())
                        .max(hold)
                        .max(self.ordinary_gold_reserve() + held)
                } else {
                    reserve
                };
                let Some(cost) = opt.buy_cost.filter(|cost| *cost > 0) else {
                    continue;
                };
                let value = self.item_value(ci, c, opt, tally);
                if value <= self.decisions.purchase_min_value {
                    continue;
                }
                if self.decisions.purchase_savings_turns > 0
                    && matches!(opt.item, ProductionItem::Building { .. })
                    && matches!(
                        opt.buy_block,
                        None | Some(legatum_core::economy::BuyBlock::NotEnoughGold)
                    )
                    && self.gold_left - cost < purchase_reserve
                    && net > 0
                {
                    let gap = cost + purchase_reserve - self.gold_left;
                    let eta = (gap + net - 1) / net;
                    if eta <= self.decisions.purchase_savings_turns && opt.turns > eta + 1 {
                        let candidate = (value, cost, purchase_reserve, eta);
                        if future.is_none_or(|f| candidate.0 > f.0) {
                            future = Some(candidate);
                        }
                    }
                }
                if opt.buy_block.is_some() || self.gold_left - cost < purchase_reserve {
                    continue;
                }
                let candidate = (
                    if self.decisions.purchase_savings_turns > 0 {
                        value
                    } else {
                        value * 1000 / cost
                    },
                    std::cmp::Reverse(cost),
                    std::cmp::Reverse(c.id),
                    opt.item,
                );
                if best.is_none_or(|best| candidate > best) {
                    best = Some(candidate);
                }
            }
        }
        if let Some((future_value, target_cost, target_reserve, eta)) = future
            && best.is_none_or(|b| future_value > b.0)
            && best.is_none_or(|b| {
                let affordable_cost = b.1.0;
                let gap_after = target_cost + target_reserve - (self.gold_left - affordable_cost);
                (gap_after + net - 1) / net > eta
            })
        {
            return;
        }
        if let Some((_, std::cmp::Reverse(cost), std::cmp::Reverse(city), item)) = best {
            self.gold_left -= cost;
            self.orders.push(Order::BuyItem { city, item });
            tally.add(item);
        } else {
            self.buy_surplus_army(tally, reserve);
            self.tag_buys("surplus_army");
        }
    }

    /// Gene `hollow_march` part 8: at war with a hollow leader whose every city the forecast
    /// reads as out of reach, buy the march force with Gold above a small reserve (the Swiss
    /// hoard savings don't block it), one unit a turn, until the fielded army meets the war
    /// quota. The
    /// unit has the best purchase score near the enemy, a shooter while the nearest
    /// target is walled and fewer than two shooters stand in the field.
    fn buy_war_army(&mut self, tally: &mut Tally) -> bool {
        // Units already fielded, not the production pipeline: a unit queued in a 6-Production
        // town arrives after the war is decided.
        let fielded = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .count() as i32;
        if self.decisions.hollow_march & crate::hollow_march::BUYS == 0
            || fielded >= self.desired_army()
            || self
                .orders
                .iter()
                .any(|o| matches!(o, Order::BuyItem { item, .. } if is_military_unit(*item)))
        {
            return false;
        }
        let Some(team) = self.hollow_war_target() else {
            return false;
        };
        if self.campaign_against(team).is_ok() {
            return false;
        }
        let v = self.view();
        let targets: Vec<&legatum_core::view::CityView> = v
            .cities
            .iter()
            .filter(|c| !c.owner.is_barbarian() && v.team_of(c.owner) == team)
            .collect();
        // Upkeep stays payable: income after the new unit's upkeep may not go negative.
        if v.economy.gold_per_turn.total - self.marginal_military_upkeep() < 0 {
            return false;
        }
        // A walled city falls only to shooters: while the field army has none, buy one.
        let shooters = self
            .ctx
            .own_units
            .iter()
            .filter(|u| {
                u.military
                    && crate::composition::shooter_only(u.kind, u.ranged)
                    && !self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
            })
            .count();
        let reserve = crate::hollow_march::WAR_BUY_RESERVE.max(self.frontline_upgrade_reserve());
        let mut best: Option<(
            i32,
            i32,
            std::cmp::Reverse<i32>,
            legatum_core::CityId,
            ProductionItem,
        )> = None;
        for c in &self.ctx.own_cities {
            let d = c.detail.as_ref().expect("own city detail");
            if d.resistance > 0
                || self.military_upgrade_hold(c)
                || self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == c.id))
            {
                continue;
            }
            let Some(nearest) = targets.iter().min_by_key(|t| t.pos.distance(c.pos)) else {
                continue;
            };
            let distance = nearest.pos.distance(c.pos);
            for o in &d.options {
                let ProductionItem::Unit { unit } = o.item else {
                    continue;
                };
                if !is_military_unit(o.item) || o.buy_block.is_some() {
                    continue;
                }
                let Some(cost) = o.buy_cost.filter(|c| *c > 0) else {
                    continue;
                };
                if self.gold_left - cost < reserve {
                    continue;
                }
                let stats = self
                    .view()
                    .rule_experiments
                    .unit_stats_for(self.faction, unit);
                let shooter = crate::composition::shooter_only(unit, stats.ranged);
                if nearest.walls && shooters < 2 && !shooter {
                    continue;
                }
                let value = self.military_purchase_score(unit, cost, c.pos) / (1 + distance / 4);
                let tier = if self.decisions.hard_purchase_scorer == 0
                    && self.decisions.named_fund > 0
                    && self.difficulty == AiDifficulty::Hard
                {
                    i32::from(data::unit_era_tier(unit))
                } else {
                    0
                };
                let key = (tier, value, std::cmp::Reverse(cost), c.id, o.item);
                if best.is_none_or(|b| key > b) {
                    best = Some(key);
                }
            }
        }
        let Some((_, _, std::cmp::Reverse(cost), city, item)) = best else {
            return false;
        };
        self.gold_left -= cost;
        self.orders.push(Order::BuyItem { city, item });
        tally.add(item);
        true
    }

    /// Nothing worth building can be bought: a treasury left idle wins nothing. Turn a
    /// real surplus into army (defense and title strength) while upkeep stays covered.
    fn buy_surplus_army(&mut self, tally: &mut Tally, reserve: i32) {
        // Surplus is routine spending: never on top of this turn's emergency purchase.
        if self
            .orders
            .iter()
            .any(|o| matches!(o, Order::BuyItem { item, .. } if is_military_unit(*item)))
        {
            return;
        }
        let upkeep = self.marginal_military_upkeep();
        // Each further 400 Gold of idle treasury funds one more unit beyond the
        // routine army: strength wins the Army title and breaks late stalemates.
        let cap = self.desired_army() + 2 + (self.gold_left - reserve).max(0) / 400;
        if tally.army >= cap {
            return;
        }
        // The added upkeep must stay payable for 60 Standard turns from income or treasury.
        let deficit = (upkeep - self.view().economy.gold_per_turn.total).max(0);
        let reserve = reserve + deficit * self.decisions.surplus_upkeep_turns;
        let mut best: Option<(
            i32,
            i32,
            std::cmp::Reverse<i32>,
            legatum_core::CityId,
            ProductionItem,
        )> = None;
        for (ci, c) in self.ctx.own_cities.iter().enumerate() {
            let d = c.detail.as_ref().expect("own city detail");
            if d.resistance > 0
                || self.military_upgrade_hold(c)
                || self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == c.id))
            {
                continue;
            }
            let exposed = self.exposed_city(c) || self.ctx.threat[ci] > 0;
            for o in &d.options {
                let ProductionItem::Unit { unit } = o.item else {
                    continue;
                };
                if !is_military_unit(o.item) || o.buy_block.is_some() {
                    continue;
                }
                let Some(cost) = o.buy_cost.filter(|c| *c > 0) else {
                    continue;
                };
                // Keep enough for the reserve plus a second purchase: this is surplus only.
                if self.gold_left - cost * self.decisions.surplus_purchase_percent / 100 < reserve {
                    continue;
                }
                let value = self.military_purchase_score(unit, cost, c.pos)
                    + if self.mongol_horde()
                        && composition::is_mounted(unit)
                        && self.best_mounted_option(c) == Some(unit)
                    {
                        1000
                    } else {
                        0
                    }
                    + if exposed { 50 } else { 0 };
                let tier = if self.decisions.hard_purchase_scorer == 0
                    && self.decisions.named_fund > 0
                    && self.difficulty == AiDifficulty::Hard
                {
                    i32::from(data::unit_era_tier(unit))
                } else {
                    0
                };
                let key = (tier, value, std::cmp::Reverse(cost), c.id, o.item);
                if best.is_none_or(|b| key > b) {
                    best = Some(key);
                }
            }
        }
        if let Some((_, _, std::cmp::Reverse(cost), city, item)) = best {
            self.gold_left -= cost;
            self.orders.push(Order::BuyItem { city, item });
            tally.add(item);
        }
    }

    /// Visible enemy military within five tiles of a city held by fewer than two units, or
    /// by less power than the visible force: buy a defender there before any optional
    /// building, and otherwise keep the gold for one (`Frontline::Save`: later purchases leave
    /// its price, they don't stop). The emergency purchase only fires once a capture is
    /// forecast, by which time one bought unit is often too late.
    fn buy_frontline_defender(&mut self, tally: &mut Tally) -> Frontline {
        // The cheapest defender a threatened city could buy, plus the reserve it must leave.
        let mut save: Option<i32> = None;
        for c in self.ctx.own_cities.clone() {
            let d = c.detail.as_ref().expect("own city detail");
            let radius = self.decisions.frontline_purchase_radius;
            // Scouts cannot take a garrisoned city; they do not warrant a purchase.
            let approaching = |e: &legatum_core::view::UnitView| {
                e.military
                    && e.kind != UnitKind::Scout
                    && e.pos.distance(c.pos) <= radius
                    && self.ctx.reachable(e.pos, c.pos)
            };
            let near_enemy = self.ctx.threat_units.iter().any(|e| approaching(e));
            let held = self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military && u.kind != UnitKind::Scout && u.pos.distance(c.pos) <= 2)
                .count();
            let outmatched = {
                let enemy: i32 = self
                    .ctx
                    .threat_units
                    .iter()
                    .filter(|e| approaching(e))
                    .map(|e| crate::analysis::threat_power(self.view(), e))
                    .sum();
                enemy > self.local_defense(c)
            };
            if !near_enemy || (held >= 2 && !outmatched) || d.resistance > 0 {
                continue;
            }
            if self.military_upgrade_hold(c) {
                continue;
            }
            // A city that already has this turn's purchase (door defense, a rescue) has its
            // defender; it holds nothing back.
            if self
                .orders
                .iter()
                .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == c.id))
            {
                continue;
            }
            // Preemptive, not a rescue: the usual reserve (Switzerland' VP goal) stands.
            let keep = ((self.marginal_military_upkeep()
                - self.view().economy.gold_per_turn.total)
                .max(0)
                * 10)
                .max(self.gold_reserve());
            let keep = keep.max(self.live_goal_floor());
            // Gene `buy_hold` 2: as readiness, unless an attack can arrive this turn.
            let keep = if self.decisions.buy_hold >= 2 && !self.defense_risk(c, None).threatened {
                keep.max(self.gold_reserve() + self.held_gold())
            } else {
                keep
            };
            // Military units this city could buy once the Gold is there: unblocked, or blocked
            // only by the treasury.
            // (unit, quote, item, buyable now)
            let quotes: Vec<(UnitKind, i32, ProductionItem, bool)> = d
                .options
                .iter()
                .filter_map(|o| {
                    let ProductionItem::Unit { unit } = o.item else {
                        return None;
                    };
                    if !is_military_unit(o.item)
                        || o.buy_block.is_some_and(|b| {
                            !matches!(b, legatum_core::economy::BuyBlock::NotEnoughGold)
                        })
                    {
                        return None;
                    }
                    let cost = o.buy_cost.filter(|c| *c > 0)?;
                    Some((unit, cost, o.item, o.buy_block.is_none()))
                })
                .collect();
            let best = quotes
                .iter()
                .filter(|&&(_, cost, _, now)| now && self.gold_left - cost >= keep)
                .map(|&(unit, cost, item, _)| {
                    (
                        if self.decisions.hard_purchase_scorer == 0
                            && self.decisions.named_fund > 0
                            && self.difficulty == AiDifficulty::Hard
                        {
                            i32::from(data::unit_era_tier(unit))
                        } else {
                            0
                        },
                        self.military_purchase_score(unit, cost, c.pos),
                        std::cmp::Reverse(cost),
                        item,
                    )
                })
                .max();
            if let Some((_, _, std::cmp::Reverse(cost), item)) = best {
                self.gold_left -= cost;
                self.orders.push(Order::BuyItem { city: c.id, item });
                tally.add(item);
                return Frontline::Bought;
            }
            if let Some(cheapest) = quotes.iter().map(|q| q.1).min() {
                save = Some(save.map_or(keep + cheapest, |s| s.min(keep + cheapest)));
            }
        }
        save.map_or(Frontline::Clear, Frontline::Save)
    }
}

/// Stateless, deterministic city production advice. The caller submits the returned item
/// as an ordinary production order; this function cannot spend Gold or reveal hidden state.
pub fn city_automation_choice(
    view: &PlayerView,
    city: CityId,
    focus: CityBuildFocus,
) -> Option<ProductionItem> {
    city_automation_choice_with_pipeline(view, city, focus, &[])
}

pub fn city_automation_choice_with_pipeline(
    view: &PlayerView,
    city: CityId,
    focus: CityBuildFocus,
    planned: &[ProductionItem],
) -> Option<ProductionItem> {
    let faction = view.players.iter().find(|p| p.id == view.me)?.faction;
    let brain =
        crate::scripted::ScriptedBrain::with_difficulty(faction, 0, view.me, AiDifficulty::Hard);
    let mut planner = crate::scripted::Planner::new(view, brain.knobs, faction, 0);
    planner.difficulty = AiDifficulty::Hard;
    planner.decisions = brain.planner_decisions();
    planner.automated_choice(city, focus, planned)
}

#[cfg(test)]
mod automation_tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::TechId;

    #[test]
    fn a_one_build_vp_can_outbid_a_routine_peacetime_queue() {
        use legatum_core::objectives::{Family, Objective};
        use legatum_core::view::{EraView, ObjectiveProgress, ObjectiveView};
        let mut view = ViewBuilder::new(20, 20)
            .known_techs(&[TechId::Chivalry])
            .city(1, ME, off(8, 8), 6)
            .build();
        view.players[0].era = 3;
        view.cities[0]
            .detail
            .as_mut()
            .unwrap()
            .buildings
            .push(BuildingKind::Walls);
        let objective = Objective::Buildings {
            building: BuildingKind::Castle,
            cities: 1,
        };
        view.eras = vec![EraView {
            era: 3,
            name: "Medieval".into(),
            objectives: vec![ObjectiveView {
                slot: 0,
                family: Family::Build,
                text: String::new(),
                target: 1,
                objective,
            }],
            entered_by: vec![ME],
            scored_by: vec![],
        }]
        .into();
        view.objective_progress = vec![ObjectiveProgress {
            era: 3,
            slot: 0,
            current: 0,
            target: 1,
            entered: true,
            era_scored: false,
            entered_turn: None,
            ready: None,
        }];
        let faction = view.faction_of(ME);
        let mut p = Planner::new(&view, crate::Knobs::for_faction(faction), faction, 7);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.cheapest_objective = 1;
        let tally = Tally {
            army: 0,
            rangers: 0,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 2,
        };
        let granary = ProductionItem::Building {
            building: BuildingKind::Granary,
        };
        let castle = ProductionItem::Building {
            building: BuildingKind::Castle,
        };
        assert_eq!(
            p.objective_production_choice(0, &view.cities[0], &tally, Some(granary)),
            Some(castle)
        );
    }

    #[test]
    fn focus_changes_hards_city_pick_and_is_deterministic() {
        let view = ViewBuilder::new(20, 20)
            .known_techs(&[TechId::Writing, TechId::Currency])
            .city(1, ME, off(8, 8), 4)
            .unit(2, ME, UnitKind::Spearman, off(8, 8))
            .build();
        let pick = |focus| city_automation_choice(&view, CityId(1), focus);
        let science = ProductionItem::Building {
            building: BuildingKind::Library,
        };
        let gold = ProductionItem::Building {
            building: BuildingKind::Market,
        };
        assert_eq!(pick(CityBuildFocus::Science), Some(science));
        assert_eq!(pick(CityBuildFocus::Gold), Some(gold));
        assert_eq!(pick(CityBuildFocus::Science), pick(CityBuildFocus::Science));
        let threatened = ViewBuilder::new(20, 20)
            .known_techs(&[TechId::Writing, TechId::Currency])
            .city(1, ME, off(8, 8), 4)
            .unit(2, ME, UnitKind::Spearman, off(8, 8))
            .unit(3, ENEMY, UnitKind::Spearman, off(10, 8))
            .build();
        assert!(
            matches!(city_automation_choice(&threatened, CityId(1), CityBuildFocus::Military), Some(item) if is_military_unit(item))
        );
    }

    /// A Military city trains a unit even in peacetime, when Hard would rather grow (user, 2026-11-01: a
    /// Military capital at population 2 built a Granary and a Farm instead).
    #[test]
    fn military_focus_trains_a_unit_in_peacetime() {
        for pop in [1, 2, 4] {
            let view = ViewBuilder::new(20, 20)
                .city(1, ME, off(8, 8), pop)
                .unit(2, ME, UnitKind::Spearman, off(8, 8))
                .build();
            assert!(
                matches!(city_automation_choice(&view, CityId(1), CityBuildFocus::Military), Some(item) if is_military_unit(item)),
                "population {pop}"
            );
        }
    }

    /// Science and Gold never train military units, even for an empty or threatened city.
    #[test]
    fn science_and_gold_never_train_military_even_in_empty_or_threatened_cities() {
        let known = [TechId::Writing, TechId::Currency];
        let empire = |garrison: bool, enemy: bool| {
            let mut b = ViewBuilder::new(30, 20)
                .known_techs(&known)
                .city(1, ME, off(8, 8), 2)
                .city(2, ME, off(14, 8), 4)
                .city(3, ME, off(20, 8), 4)
                .city(4, ME, off(8, 14), 4);
            if garrison {
                b = b.unit(10, ME, UnitKind::Spearman, off(8, 8));
            }
            if enemy {
                b = b.unit(11, ENEMY, UnitKind::Spearman, off(10, 8));
            }
            b.build()
        };
        let science = ProductionItem::Building {
            building: BuildingKind::Library,
        };
        let gold = ProductionItem::Building {
            building: BuildingKind::Market,
        };
        let pick = |view: &PlayerView, focus| city_automation_choice(view, CityId(1), focus);
        let held = empire(true, false);
        assert_eq!(pick(&held, CityBuildFocus::Science), Some(science));
        assert_eq!(pick(&held, CityBuildFocus::Gold), Some(gold));
        let empty = empire(false, false);
        assert_eq!(pick(&empty, CityBuildFocus::Science), Some(science));
        assert_eq!(pick(&empty, CityBuildFocus::Gold), Some(gold));
        let besieged = empire(true, true);
        assert_eq!(pick(&besieged, CityBuildFocus::Science), Some(science));
        assert_eq!(pick(&besieged, CityBuildFocus::Gold), Some(gold));
    }
}

/// What the frontline check (`buy_frontline_defender`) decided for this turn's purchases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Frontline {
    /// It bought a defender: this turn's routine purchases stop.
    Bought,
    /// A threatened city can't yet afford its defender: later purchases leave this much Gold
    /// (the reserve plus the cheapest such defender), so the treasury still saves for it.
    Save(i32),
    /// No threatened city wants a purchase it can make.
    Clear,
}

#[cfg(test)]
mod bombardment_purchase_tests {
    use super::*;
    use crate::{
        Knobs,
        fixture::{ENEMY, ME, ViewBuilder},
    };
    use legatum_core::Hex;

    #[test]
    fn roadworks_value_prioritizes_warlike_factions_and_a_campaign_target_link() {
        use crate::campaign::Campaign;
        use legatum_core::{CityId, UnitId};

        let view = ViewBuilder::new(30, 20)
            .city(1, ME, Hex::new(5, 5), 3)
            .city(2, ENEMY, Hex::new(13, 5), 3)
            .build();
        let own_city = &view.cities[0];
        let mut sparta = Planner::new(
            &view,
            Knobs::for_faction(Faction::Sparta),
            Faction::Sparta,
            1,
        );
        let sparta_value = sparta.roadworks_value(own_city);
        let peaceful = Planner::new(
            &view,
            Knobs::for_faction(Faction::Byzantium),
            Faction::Byzantium,
            1,
        );
        assert!(sparta_value > peaceful.roadworks_value(own_city));

        sparta.campaigns.active = Some(Campaign {
            target: CityId(2),
            start_turn: 1,
            committed: vec![UnitId(1)],
            deadline: None,
            expected_value: 0,
            target_was_unwalled: true,
        });
        assert!(sparta.roadworks_value(own_city) >= sparta_value + 300);
    }

    #[test]
    fn switzerland_buy_the_next_work_prerequisite_after_banking_the_goal() {
        let mut view = ViewBuilder::new(24, 16)
            .faction(Faction::Switzerland)
            .known_techs(&legatum_core::TechId::ALL)
            .gold(300)
            .city(1, ME, Hex::from_offset(5, 5), 8)
            .build();
        view.turn = 70;
        view.turn_limit = None;
        view.cities[0]
            .detail
            .as_mut()
            .unwrap()
            .options
            .retain(|option| !legatum_core::data::is_great_work_item(option.item));
        let initial = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        let (city, building) = initial.work_plan.as_ref().unwrap().steps[0];
        let item = ProductionItem::Building { building };
        let detail = view.cities[0].detail.as_mut().unwrap();
        detail.options.retain(|option| option.item == item);
        detail.options[0].buy_cost = Some(100);
        detail.options[0].turns = 8;

        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        let mut tally = Tally {
            army: 0,
            rangers: 0,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 0,
        };
        planner.plan_buy(&mut tally);
        assert!(planner.orders.contains(&Order::BuyItem { city, item }));
        assert_eq!(planner.gold_left, 200);
    }

    #[test]
    fn readiness_funds_two_fronts_once_each_without_overspending_or_replacing_builds() {
        let mut view = ViewBuilder::new(36, 20)
            .city(1, ME, Hex::new(6, 6), 3)
            .city(2, ME, Hex::new(16, 6), 3)
            .unit(50, ENEMY, UnitKind::Spearman, Hex::new(10, 6))
            .unit(51, ENEMY, UnitKind::Spearman, Hex::new(20, 6))
            .build();
        for c in &mut view.cities {
            let d = c.detail.as_mut().unwrap();
            d.producing = Some(ProductionItem::Building {
                building: BuildingKind::Market,
            });
            d.options.retain(|o| {
                o.item
                    == (ProductionItem::Unit {
                        unit: UnitKind::Spearman,
                    })
            });
            d.options[0].buy_cost = Some(100);
        }
        for (gold, expected) in [(150, 1), (250, 2)] {
            view.economy.gold = gold;
            let mut p = Planner::new(
                &view,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                7,
            );
            // Readiness funding itself; gene buy_hold 2+ (default 3) holds Gold from it and has its own tests.
            p.decisions.buy_hold = 0;
            p.decisions.named_fund = 0;
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.prepare_hard_defense();
            p.buy_ready_defenders();
            p.buy_ready_defenders();
            assert_eq!(p.orders.len(), expected);
            assert!(p.orders.iter().all(|o| matches!(o, Order::BuyItem { .. })));
            assert_eq!(p.gold_left, gold - 100 * expected as i32);
        }
        view.players[0].faction_goal_done = false;
        view.economy.gold = 250;
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        p.difficulty = legatum_core::AiDifficulty::Hard;
        p.decisions.named_fund = 0;
        p.prepare_hard_defense();
        p.buy_ready_defenders();
        assert!(
            p.orders.is_empty(),
            "optional readiness must bank the treasury goal first"
        );
    }

    #[test]
    fn hard_germany_prices_production_building_food_and_protects_small_cities() {
        let mut view = ViewBuilder::new(24, 16)
            .faction(Faction::Germany)
            .city(1, ME, Hex::new(5, 5), 1)
            .build();
        let mut option = view.cities[0].detail.as_ref().unwrap().options[0].clone();
        option.item = ProductionItem::Building {
            building: BuildingKind::Workshop,
        };
        option.yields.total.food = 4;
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Germany),
            Faction::Germany,
            7,
        );
        planner.difficulty = legatum_core::AiDifficulty::Hard;
        let scarce = planner.german_industry_food_cost(&view.cities[0], &option);
        option.yields.total.food = 12;
        assert_eq!(
            scarce,
            2 * planner.german_industry_food_cost(&view.cities[0], &option)
        );
        option.yields.total.food = 2;
        assert_eq!(
            planner.german_industry_food_cost(&view.cities[0], &option),
            10_000
        );
        planner.decisions.german_industry_food = 0;
        assert_eq!(
            planner.german_industry_food_cost(&view.cities[0], &option),
            0
        );
        option.item = ProductionItem::Building {
            building: BuildingKind::Forge,
        };
        assert_eq!(
            planner.legacy_foundry_food_cost(BuildingKind::Forge),
            2 * (100 + planner.knobs.expansion * 2)
        );
        view.rule_experiments
            .faction_rules_off
            .insert(legatum_core::experiments::FactionRule::GermanyIndustryFood);
        let planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Germany),
            Faction::Germany,
            7,
        );
        assert_eq!(
            planner.german_industry_food_cost(&view.cities[0], &option),
            0
        );
        assert_eq!(planner.legacy_foundry_food_cost(BuildingKind::Forge), 0);
    }

    #[test]
    fn capture_alarm_requires_an_enemy_but_preserves_local_emergencies() {
        let mut view = ViewBuilder::new(30, 18)
            .city(1, ME, Hex::new(5, 5), 3)
            .city(2, ME, Hex::new(12, 5), 2)
            .gold(0)
            .build();
        let d = view.cities[0].detail.as_mut().unwrap();
        d.producing = Some(ProductionItem::Building {
            building: BuildingKind::Granary,
        });
        d.turns_to_complete = Some(10);
        view.cities[1].hp = 0;
        view.cities[1].detail.as_mut().unwrap().resistance = 3;
        let switches = |v: &legatum_core::PlayerView, gene, difficulty| {
            let mut p = Planner::new(v, Knobs::for_faction(Faction::Germany), Faction::Germany, 7);
            p.difficulty = difficulty;
            p.decisions.capture_alarm = gene;
            p.plan_emergency_production();
            p.orders.iter().any(|o| {
                matches!(o, Order::SetProduction { city, item }
                if city.0 == 1 && is_military_unit(*item))
            })
        };
        assert!(switches(&view, 0, legatum_core::AiDifficulty::Hard));
        assert!(!switches(&view, 1, legatum_core::AiDifficulty::Hard));
        assert!(switches(&view, 1, legatum_core::AiDifficulty::Normal));
        // An enemy near the capture retains relief builds, even far from home.
        let threatened = ViewBuilder { view: view.clone() }
            .unit(50, ENEMY, UnitKind::Spearman, Hex::new(15, 5))
            .build();
        assert!(switches(&threatened, 1, legatum_core::AiDifficulty::Hard));
        // Local bombardment still interrupts economy without the neighboring alarm.
        let direct = ViewBuilder { view }
            .unit(50, ENEMY, UnitKind::Archer, Hex::new(7, 5))
            .build();
        assert!(switches(&direct, 1, legatum_core::AiDifficulty::Hard));
    }

    #[test]
    fn emergency_gene_ignores_a_standing_shooter_want_without_bombardment() {
        let home = Hex::new(8, 8);
        let granary = ProductionItem::Building {
            building: BuildingKind::Granary,
        };
        let build = |enemies: &[(u32, UnitKind, Hex)]| {
            let mut b = ViewBuilder::new(24, 16)
                .city(1, ME, home, 2)
                .city(2, ME, Hex::new(20, 3), 2)
                .gold(0)
                .unit(1, ME, UnitKind::Spearman, home);
            for (id, kind, at) in enemies {
                b = b.unit(*id, ENEMY, *kind, *at);
            }
            let mut view = b.build();
            let d = view.cities[0].detail.as_mut().unwrap();
            d.producing = Some(granary);
            d.turns_to_complete = Some(10);
            view
        };
        let fires = |view: &legatum_core::PlayerView, gene| {
            let mut p = Planner::new(
                view,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                7,
            );
            p.decisions.emergency_bombardment_only = gene;
            p.decisions.production_threat_agreement = 0;
            p.plan_emergency_production();
            p.orders
                .iter()
                .any(|o| matches!(o, Order::SetProduction { item, .. } if is_military_unit(*item)))
        };
        let standing = build(&[(50, UnitKind::Spearman, Hex::new(11, 8))]);
        assert!(fires(&standing, 0), "off: the standing want interrupts");
        assert!(!fires(&standing, 1), "on: it does not");
        let bombarded = build(&[(50, UnitKind::Archer, Hex::new(10, 8))]);
        assert!(fires(&bombarded, 0));
        assert!(fires(&bombarded, 1), "bombardment still interrupts");
        let swarm = build(&[
            (50, UnitKind::Spearman, Hex::new(10, 8)),
            (51, UnitKind::Spearman, Hex::new(9, 8)),
            (52, UnitKind::Spearman, Hex::new(9, 9)),
            (53, UnitKind::Spearman, Hex::new(10, 9)),
        ]);
        assert!(fires(&swarm, 1), "threat above defence still interrupts");
    }

    #[test]
    fn walled_city_shooter_need_does_not_flip_back_to_spearman() {
        // India capital, Rome 8.9.71 T27: a Spearman garrison, no shooter and
        // six visible Spearmen three hexes out. Reprice only the production
        // choice; the siege matchup and old bombardment genes are explicit.
        let home = Hex::new(8, 8);
        let spear = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        let archer = ProductionItem::Unit {
            unit: UnitKind::Archer,
        };
        let mut view = ViewBuilder::new(24, 18)
            .faction(Faction::India)
            .city(1, ME, home, 3)
            .walled(1)
            .city(2, ME, Hex::new(20, 5), 2)
            .unit(1, ME, UnitKind::Spearman, home)
            .unit(50, ENEMY, UnitKind::Spearman, Hex::new(11, 8))
            .unit(51, ENEMY, UnitKind::Spearman, Hex::new(8, 11))
            .unit(52, ENEMY, UnitKind::Spearman, Hex::new(11, 7))
            .unit(53, ENEMY, UnitKind::Spearman, Hex::new(10, 9))
            .unit(54, ENEMY, UnitKind::Spearman, Hex::new(7, 11))
            .unit(55, ENEMY, UnitKind::Spearman, Hex::new(9, 10))
            .build();
        view.turn = 27;
        let d = view.cities[0].detail.as_mut().unwrap();
        d.producing = Some(archer);
        d.turns_to_complete = Some(8);
        d.options.retain(|o| o.item == spear || o.item == archer);
        let tally = Tally {
            army: 3,
            rangers: 2,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 0,
        };
        let pick = |view: &legatum_core::PlayerView, gene| {
            let mut p = Planner::new(view, Knobs::for_faction(Faction::India), Faction::India, 7);
            p.difficulty = AiDifficulty::Hard;
            p.decisions.production_threat_agreement = gene;
            p.decisions.emergency_bombardment_only = 0;
            p.decisions.army_door_production = 0;
            p.choose_item(0, &view.cities[0], &tally)
        };
        let mut before_emergency = view.clone();
        before_emergency.cities[0]
            .detail
            .as_mut()
            .unwrap()
            .producing = Some(spear);
        let mut p = Planner::new(
            &before_emergency,
            Knobs::for_faction(Faction::India),
            Faction::India,
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p.decisions.production_threat_agreement = 0;
        p.decisions.emergency_bombardment_only = 0;
        p.decisions.army_door_production = 0;
        p.plan_emergency_production();
        assert!(
            p.orders.contains(&Order::SetProduction {
                city: before_emergency.cities[0].id,
                item: archer,
            }),
            "old emergency pass starts the Archer"
        );
        assert_eq!(
            pick(&view, 0),
            Some(spear),
            "old routine pass undoes the Archer"
        );
        assert_eq!(
            pick(&view, 1),
            Some(archer),
            "routine keeps the useful shooter"
        );
    }

    #[test]
    fn visible_threat_approach_does_not_resume_mongol_settler() {
        // Rome 8.9.67 T42–T44: the threat pass started an Archer in an open
        // Mongol city, then routine resumed a Settler although the human army
        // was visible at the border. The threat model can read this before war.
        let home = Hex::new(8, 8);
        let ranger = ProductionItem::Unit {
            unit: UnitKind::Archer,
        };
        let settler = ProductionItem::Unit {
            unit: UnitKind::Settler,
        };
        let mut view = ViewBuilder::new(24, 18)
            .faction(Faction::Mongols)
            .city(1, ME, home, 4)
            .city(2, ENEMY, Hex::new(12, 8), 2)
            .unit(1, ME, UnitKind::Spearman, home)
            .unit(50, ENEMY, UnitKind::Spearman, Hex::new(11, 8))
            .unit(51, ENEMY, UnitKind::Spearman, Hex::new(12, 7))
            .build();
        view.turn = 43;
        view.diplomacy.legacy = false;
        let d = view.cities[0].detail.as_mut().unwrap();
        d.producing = Some(ranger);
        d.turns_to_complete = Some(6);
        d.options.retain(|o| o.item == settler || o.item == ranger);
        let mut model = crate::opponents::OpponentAssessments::default();
        let record = model.threats.records.entry(ENEMY).or_default();
        record.captures = vec![35, 37, 39, 41];
        record.eliminations = vec![legatum_core::PlayerId(3)];
        model.update(&view);
        let tally = Tally {
            army: 3,
            rangers: 1,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 1,
        };
        let pick = |gene| {
            let mut p = Planner::new(
                &view,
                Knobs::for_faction(Faction::Mongols),
                Faction::Mongols,
                7,
            );
            p.difficulty = AiDifficulty::Hard;
            p.opponents = Some(&model);
            p.decisions.production_threat_agreement = gene;
            p.decisions.emergency_bombardment_only = 0;
            p.decisions.army_door_production = 0;
            assert!(p.threat_exposure(&view.cities[0]) > 0);
            p.choose_item(0, &view.cities[0], &tally)
        };
        assert_eq!(pick(0), Some(settler));
        assert_eq!(pick(1), Some(ranger));
    }

    #[test]
    fn choosing_counts_the_active_builds_stored_production() {
        let mut view = ViewBuilder::new(24, 16)
            .faction(Faction::India)
            .known_techs(&legatum_core::TechId::ALL)
            .city(1, ME, Hex::new(4, 4), 3)
            .city(2, ME, Hex::new(12, 4), 3)
            .build();
        view.turn = 30;
        let tally = Tally {
            army: 1,
            rangers: 0,
            scouts: 1,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 0,
        };
        let soldier = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        let ranger = ProductionItem::Unit {
            unit: UnitKind::Archer,
        };
        let pick = |view: &legatum_core::PlayerView| {
            let mut p = Planner::new(view, Knobs::for_faction(Faction::India), Faction::India, 7);
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.choose_item_inner(1, &view.cities[1], &tally)
        };
        let mut view = view;
        view.cities[1]
            .detail
            .as_mut()
            .unwrap()
            .options
            .retain(|o| o.item == soldier || o.item == ranger);
        let first = pick(&view).expect("a pick");
        let other = if first == soldier { ranger } else { soldier };
        let mut under_way = view.clone();
        let d = under_way.cities[1].detail.as_mut().unwrap();
        d.producing = Some(other);
        for o in &mut d.options {
            if o.item == other {
                o.production_stored = o.cost * 3 / 4;
                o.production_banked = 0;
            }
        }
        assert_eq!(
            pick(&under_way),
            Some(other),
            "the item under way keeps its stored share"
        );
    }

    #[test]
    fn nearby_shooters_out_of_range_do_not_veto_ranged_relief() {
        let home = Hex::new(8, 8);
        let mut view = ViewBuilder::new(24, 16)
            .city(1, ME, home, 2)
            .gold(700)
            .unit_hp(1, ME, UnitKind::Spearman, home, 20)
            .unit(2, ME, UnitKind::Archer, Hex::new(5, 8))
            .unit(50, ENEMY, UnitKind::Archer, Hex::new(10, 8))
            .build();
        view.cities[0]
            .detail
            .as_mut()
            .unwrap()
            .options
            .iter_mut()
            .find(|o| {
                o.item
                    == (ProductionItem::Unit {
                        unit: UnitKind::Archer,
                    })
            })
            .unwrap()
            .buy_cost = Some(100);
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        p.buy_emergency_defender();
        assert!(
            p.orders.iter().any(|o| matches!(
                o,
                Order::BuyItem {
                    item: ProductionItem::Unit {
                        unit: UnitKind::Archer
                    },
                    ..
                }
            )),
            "{:?}",
            p.orders
        );
        view.economy.gold = 0;
        let d = view.cities[0].detail.as_mut().unwrap();
        d.producing = Some(ProductionItem::Unit {
            unit: UnitKind::Spearman,
        });
        d.turns_to_complete = Some(20);
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        p.plan_emergency_production();
        assert!(
            p.orders.iter().any(|o| matches!(
                o,
                Order::SetProduction {
                    item: ProductionItem::Unit {
                        unit: UnitKind::Archer
                    },
                    ..
                }
            )),
            "{:?}",
            p.orders
        );
    }
}

#[cfg(test)]
mod rush_tests {
    use super::*;
    use crate::fixture::{ME, ViewBuilder, off};
    use crate::{Knobs, Strategy};
    use legatum_core::{Faction, PlayerView, TechId};

    fn rush_planner(view: &PlayerView) -> Planner<'_> {
        let mut p = crate::fixture::planner(view, 1);
        p.knobs = Strategy::Rush.knobs();
        p
    }

    #[test]
    fn opening_market_prevents_insolvency_without_delaying_a_funded_army() {
        use crate::openings::{Opening, OpeningPlan, OpeningPolicy, Phase};
        let mut view = ViewBuilder::new(20, 20)
            .known_techs(&[TechId::Irrigation, TechId::Currency])
            .city(1, ME, off(5, 5), 2)
            .unit(1, ME, UnitKind::Spearman, off(5, 5))
            .unit(2, ME, UnitKind::Spearman, off(5, 6))
            .build();
        let soldier = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        let market = ProductionItem::Building {
            building: BuildingKind::Market,
        };
        let detail = view.cities[0].detail.as_mut().unwrap();
        detail.producing = Some(soldier);
        detail.production_stored = 30;
        detail.turns_to_complete = Some(5);
        view.economy.gold_per_turn.total = 0;
        let policy = OpeningPolicy::builtin(Opening::Rush);
        for (gold, redirect) in [(0, true), (1000, false)] {
            view.economy.gold = gold;
            let mut p = Planner::new(
                &view,
                Knobs::for_faction(view.faction_of(ME)),
                view.faction_of(ME),
                7,
            );
            p.fund_rush_army();
            assert!(p.orders.is_empty(), "Default keeps its investment policy");
            p.opening = Some(OpeningPlan {
                investment: None,
                policy: &policy,
                phase: Phase::Opening,
                next_build: Some(soldier),
            });
            p.fund_rush_army();
            assert_eq!(
                p.orders
                    .iter()
                    .any(|o| matches!(o, Order::SetProduction { item, .. } if *item == market)),
                redirect
            );
        }
    }

    #[test]
    fn rush_opening_funds_production_and_income_before_the_mixed_force() {
        let mut view = ViewBuilder::new(20, 20)
            .known_techs(&[
                TechId::BronzeWorking,
                TechId::Masonry,
                TechId::Feudalism,
                TechId::Irrigation,
                TechId::Currency,
            ])
            .city(1, ME, off(5, 5), 3)
            .build();
        let mut tally = Tally {
            army: 2,
            rangers: 0,
            scouts: 1,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 1,
        };
        for building in [BuildingKind::Workshop, BuildingKind::Market] {
            let mut p = rush_planner(&view);
            p.strategy = Some(Strategy::Rush);
            assert_eq!(
                p.rush_opening_item(&view.cities[0], &tally),
                Some(ProductionItem::Building { building })
            );
            view.cities[0]
                .detail
                .as_mut()
                .unwrap()
                .buildings
                .push(building);
        }
        let mut p = rush_planner(&view);
        p.strategy = Some(Strategy::Rush);
        assert_eq!(
            p.rush_opening_item(&view.cities[0], &tally),
            Some(ProductionItem::Unit {
                unit: UnitKind::Archer
            })
        );
        tally.rangers = 2;
        tally.army = 4;
        assert_eq!(
            p.rush_opening_item(&view.cities[0], &tally),
            Some(ProductionItem::Unit {
                unit: UnitKind::Pikeman
            })
        );
        tally.army = 5;
        assert_eq!(p.rush_opening_item(&view.cities[0], &tally), None);
    }

    #[test]
    fn build_overlay_stops_troops_beyond_the_garrison_in_a_quiet_city() {
        let mut view = ViewBuilder::new(20, 20)
            .known_techs(&[TechId::BronzeWorking, TechId::Masonry])
            .city(1, ME, off(5, 5), 3)
            .build();
        view.turn = 122;
        let tally = Tally {
            army: 1,
            rangers: 0,
            scouts: 1,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 1,
        };
        let c = &view.cities[0];
        let soldier = |posture| {
            let mut p = Planner::new(
                &view,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                1,
            );
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.campaigns.plan.posture = posture;
            let opt = c
                .detail
                .as_ref()
                .unwrap()
                .options
                .iter()
                .find(|o| {
                    o.item
                        == ProductionItem::Unit {
                            unit: UnitKind::Spearman,
                        }
                })
                .unwrap();
            p.item_value(0, c, opt, &tally)
        };
        assert_eq!(soldier(Some(crate::plan::Posture::Build)), 0);
        assert!(
            soldier(None) > 0,
            "without the overlay the quota still wants troops"
        );
    }

    #[test]
    fn victory_overlay_runs_the_title_planner_from_the_first_turn() {
        let view = ViewBuilder::new(20, 20).city(1, ME, off(5, 5), 3).build();
        let mut p = Planner::new(&view, Knobs::for_faction(Faction::India), Faction::India, 1);
        p.difficulty = legatum_core::AiDifficulty::Hard;
        assert!(!p.pursuing_victory());
        p.campaigns.plan.posture = Some(crate::plan::Posture::Victory);
        assert!(p.pursuing_victory());
    }

    #[test]
    fn rush_reserves_affordable_force_slots_for_ranged_support() {
        let mut view = ViewBuilder::new(20, 20)
            .known_techs(&[TechId::BronzeWorking, TechId::Masonry, TechId::Feudalism])
            .city(1, ME, off(5, 5), 3)
            .build();
        view.turn = 122;
        let mut p = rush_planner(&view);
        p.strategy = Some(Strategy::Rush);
        let tally = Tally {
            army: 3,
            rangers: 0,
            scouts: 1,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 1,
        };
        let c = &view.cities[0];
        let ci = 0;
        // Past the scripted opening, the affordable quota still reserves ranged slots.
        let value = |unit| {
            p.item_value(
                ci,
                c,
                c.detail
                    .as_ref()
                    .unwrap()
                    .options
                    .iter()
                    .find(|o| o.item == ProductionItem::Unit { unit })
                    .unwrap(),
                &tally,
            )
        };
        assert_eq!(p.desired_army(), 6);
        assert_eq!(value(UnitKind::Spearman), 0);
        assert!(value(UnitKind::Archer) > 0);
        let ordinary = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            1,
        );
        assert_eq!(ordinary.strategy, None);
    }
}

#[cfg(test)]
mod india_office_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ME, ViewBuilder, off};

    #[test]
    fn production_focus_forecast_counts_only_gold_remitted_to_the_treasury() {
        let mut view = ViewBuilder::new(24, 16)
            .faction(Faction::India)
            .city(1, ME, off(4, 4), 3)
            .city(2, ME, off(9, 4), 3)
            .build();
        let detail = view.cities[1].detail.as_mut().unwrap();
        let item = ProductionItem::Building {
            building: BuildingKind::Granary,
        };
        detail
            .options
            .iter_mut()
            .find(|o| o.item == item)
            .unwrap()
            .yields
            .total
            .gold = 8;
        let delta = |view: &legatum_core::PlayerView| {
            let mut planner =
                Planner::new(view, Knobs::for_faction(Faction::India), Faction::India, 7);
            planner.orders.push(Order::SetProduction {
                city: legatum_core::CityId(2),
                item,
            });
            planner.campaign_income_delta()
        };
        assert_eq!(delta(&view), 0);
        view.cities[1]
            .detail
            .as_mut()
            .unwrap()
            .buildings
            .push(BuildingKind::Market);
        assert_eq!(delta(&view), 3);
    }

    #[test]
    fn safe_provincial_office_precedes_the_opening_queue_with_one_guard() {
        let mut view = ViewBuilder::new(24, 16)
            .faction(Faction::India)
            .known_techs(&legatum_core::TechId::ALL)
            .city(1, ME, off(4, 4), 3)
            .city(2, ME, off(12, 4), 3)
            .build();
        view.turn = 30;
        let city = &mut view.cities[1];
        let office = ProductionItem::Building {
            building: BuildingKind::Market,
        };
        city.detail
            .as_mut()
            .unwrap()
            .options
            .iter_mut()
            .find(|o| o.item == office)
            .unwrap()
            .yields
            .total
            .gold = 2;
        let tally = Tally {
            army: 1,
            rangers: 0,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 1,
        };
        let mut planner =
            Planner::new(&view, Knobs::for_faction(Faction::India), Faction::India, 7);
        planner.difficulty = legatum_core::AiDifficulty::Hard;
        planner.decisions.india_early_office_min_gold = 2;
        assert_eq!(
            planner.choose_item_inner(1, &view.cities[1], &tally),
            Some(office)
        );
    }

    #[test]
    fn provincial_office_values_remitted_gold_and_diwan_preparation() {
        let mut view = ViewBuilder::new(24, 16)
            .faction(Faction::India)
            .city(1, ME, off(4, 4), 3)
            .city(2, ME, off(9, 4), 3)
            .build();
        view.turn = 70;
        view.players[0].faction_goal_done = false;
        let city = &view.cities[1];
        let tally = Tally {
            army: 3,
            rangers: 1,
            scouts: 1,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 0,
        };
        let mut office = city.detail.as_ref().unwrap().options[0].clone();
        office.item = ProductionItem::Building {
            building: BuildingKind::Market,
        };
        office.yields.total.gold = 4;
        let mut planner =
            Planner::new(&view, Knobs::for_faction(Faction::India), Faction::India, 7);
        planner.difficulty = legatum_core::AiDifficulty::Hard;
        planner.decisions.india_office_goal_value = 0;
        planner.decisions.india_last_office_value = 0;
        let ordinary = planner.item_base_value(1, city, &office, &tally);
        office.yields.total.gold = 6;
        assert_eq!(
            planner.item_base_value(1, city, &office, &tally) - ordinary,
            400
        );
        planner.decisions.india_office_goal_value = 800;
        assert_eq!(
            planner.item_base_value(1, city, &office, &tally) - ordinary,
            1200
        );
        planner.decisions.india_last_office_value = 1200;
        assert_eq!(
            planner.item_base_value(1, city, &office, &tally) - ordinary,
            1800
        );
    }

    #[test]
    fn peaceful_sparta_border_can_fund_an_anticipatory_guard() {
        let mut view = ViewBuilder::new(24, 16)
            .faction(Faction::India)
            .city(1, ME, off(6, 6), 3)
            .city(2, crate::fixture::ENEMY, off(12, 6), 3)
            .gold(0)
            .build();
        view.players[crate::fixture::ENEMY.0 as usize].faction = Faction::Sparta;
        view.economy.gold_per_turn.total = -5;
        let city = &view.cities[0];
        let mut planner =
            Planner::new(&view, Knobs::for_faction(Faction::India), Faction::India, 7);
        planner.difficulty = legatum_core::AiDifficulty::Hard;
        assert!(planner.assessed_border_threat(city.pos));
        planner.decisions.india_prewar_guard = 0;
        assert!(!planner.funding_front(0, city));
        assert!(planner.troop_unfunded(0, city, UnitKind::Spearman));
        planner.decisions.india_prewar_guard = 1;
        assert!(planner.funding_front(0, city));
        assert!(!planner.troop_unfunded(0, city, UnitKind::Spearman));
    }

    #[test]
    fn threatened_india_city_can_train_a_defender_with_short_runway() {
        let mut view = ViewBuilder::new(24, 16)
            .faction(Faction::India)
            .city(1, ME, off(6, 6), 3)
            .unit(5, crate::fixture::ENEMY, UnitKind::Spearman, off(8, 6))
            .gold(0)
            .build();
        view.economy.gold_per_turn.total = -5;
        let city = &view.cities[0];
        let mut planner =
            Planner::new(&view, Knobs::for_faction(Faction::India), Faction::India, 7);
        planner.difficulty = legatum_core::AiDifficulty::Hard;
        assert!(planner.funding_front(0, city));
        planner.decisions.india_emergency_defense = 0;
        assert!(planner.troop_unfunded(0, city, UnitKind::Spearman));
        planner.decisions.india_emergency_defense = 1;
        assert!(!planner.troop_unfunded(0, city, UnitKind::Spearman));
    }
}

#[cfg(test)]
mod insolvency_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ME, ViewBuilder, off};
    use legatum_core::forecast::BankruptcyForecast;
    use legatum_core::{Breakdown, PlayerView, UnitId};

    /// Mongols (1 Gold upkeep per unit, as in rush-defense) with six military units: a
    /// Spearman garrisoning a city still in resistance (just captured), three Spearmen and a
    /// Archer in the field, and a Scout.
    fn bankrupt(deficit: i32) -> PlayerView {
        let mut v = ViewBuilder::new(24, 16)
            .faction(Faction::Mongols)
            .city_with_detail(1, ME, off(4, 4), 2)
            .city_with_detail(2, ME, off(12, 8), 1)
            .unit_hp(1, ME, UnitKind::Spearman, off(12, 8), 64)
            .unit(2, ME, UnitKind::Spearman, off(8, 8))
            .unit(3, ME, UnitKind::Spearman, off(9, 9))
            .unit(4, ME, UnitKind::Archer, off(10, 8))
            .unit(5, ME, UnitKind::Spearman, off(6, 6))
            .unit(6, ME, UnitKind::Scout, off(2, 2))
            .gold(0)
            .build();
        v.cities[1].hp = 5;
        v.cities[1].detail.as_mut().unwrap().resistance = 4;
        v.economy.bankruptcy = Some(BankruptcyForecast {
            gold: Breakdown {
                total: -deficit,
                parts: Vec::new(),
            },
            disband: None,
        });
        v
    }

    fn disbanded(v: &PlayerView) -> Vec<UnitId> {
        let mut p = Planner::new(v, Knobs::for_faction(v.faction_of(ME)), v.faction_of(ME), 7);
        p.disband_insolvent_unit();
        p.orders
            .iter()
            .filter_map(|o| match o {
                Order::DisbandUnit { unit } => Some(*unit),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_one_gold_deficit_costs_the_least_valuable_unit_never_a_captured_citys_garrison() {
        // The Wayfinder costs a Spearman's 40 but fights worst.
        assert_eq!(disbanded(&bankrupt(1)), vec![UnitId(6)]);
    }

    #[test]
    fn a_deficit_worth_several_units_is_covered_by_the_least_useful_not_left_to_the_factory() {
        // The engine would take the costliest unit and, at the same deficit, another one
        // next turn. Cover 3 Gold with the three least valuable units; the captured city's
        // garrison stays.
        let cut = disbanded(&bankrupt(3));
        assert_eq!(cut.len(), 3);
        assert!(cut.contains(&UnitId(6)));
        assert!(!cut.contains(&UnitId(1)));
    }

    #[test]
    fn the_garrison_of_a_city_in_resistance_is_worth_its_city() {
        let v = bankrupt(1);
        let p = Planner::new(
            &v,
            Knobs::for_faction(v.faction_of(ME)),
            v.faction_of(ME),
            7,
        );
        let garrison = p.ctx.own_units.iter().find(|u| u.id == UnitId(1)).unwrap();
        let field = p.ctx.own_units.iter().find(|u| u.id == UnitId(2)).unwrap();
        assert!(p.unit_keep_value(garrison) > 2 * p.unit_keep_value(field));
    }
}

#[cfg(test)]
mod swiss_market_priority_tests {
    use super::*;
    use crate::fixture::{ME, ViewBuilder, off};
    use crate::{DecisionParams, Knobs};

    #[test]
    fn market_priority_only_applies_to_a_swiss_with_a_pending_goal() {
        let mut view = ViewBuilder::new(20, 20)
            .faction(Faction::Switzerland)
            .city(1, ME, off(5, 5), 2)
            .build();
        view.players[0].faction_goal_done = false;
        let params = DecisionParams {
            swiss_market_priority: 300,
            ..DecisionParams::default()
        };
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        planner.decisions = params;
        assert_eq!(planner.swiss_market_priority(), 0);
        planner.difficulty = legatum_core::AiDifficulty::Hard;
        assert_eq!(planner.swiss_market_priority(), 300);

        view.players[0].faction_goal_done = true;
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        planner.decisions.swiss_market_priority = 300;
        assert_eq!(planner.swiss_market_priority(), 0);

        view.players[0].faction_goal_done = false;
        let mut planner =
            Planner::new(&view, Knobs::for_faction(Faction::India), Faction::India, 7);
        planner.decisions.swiss_market_priority = 300;
        assert_eq!(planner.swiss_market_priority(), 0);
    }
}

#[cfg(test)]
mod settler_buy_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ME, ViewBuilder, off};
    use legatum_core::{CityId, Faction};

    fn settler() -> ProductionItem {
        ProductionItem::Unit {
            unit: UnitKind::Settler,
        }
    }

    fn orders_after(population: i32) -> Vec<Order> {
        let view = ViewBuilder::new(20, 20)
            .city(1, ME, off(5, 5), population)
            .city(2, ME, off(12, 5), 3)
            .build();
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Mongols),
            Faction::Mongols,
            1,
        );
        p.orders = vec![
            Order::BuyItem {
                city: CityId(1),
                item: settler(),
            },
            Order::SetProduction {
                city: CityId(1),
                item: settler(),
            },
            Order::SetProduction {
                city: CityId(2),
                item: settler(),
            },
        ];
        p.drop_settler_set_after_settler_buy();
        p.orders
    }

    #[test]
    fn a_bought_settler_leaves_no_second_settler_to_set_in_a_city_it_leaves_too_small() {
        // Population 2 pays a population for the Settler and drops to 1: the follow-up is invalid.
        let small = orders_after(2);
        assert!(!small.contains(&Order::SetProduction {
            city: CityId(1),
            item: settler()
        }));
        // A city of 3 can still build the next one, and other cities are untouched.
        let big = orders_after(3);
        assert!(big.contains(&Order::SetProduction {
            city: CityId(1),
            item: settler()
        }));
        assert!(small.contains(&Order::SetProduction {
            city: CityId(2),
            item: settler()
        }));
    }
}

#[cfg(test)]
mod raze_tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder};
    use legatum_core::Hex;

    #[test]
    fn captured_city_facing_former_owners_counterattack_is_razed_but_safe_city_is_kept() {
        let pos = Hex::new(8, 8);
        let mut view = ViewBuilder::new(20, 18)
            .city(1, ME, pos, 3)
            .unit(20, ENEMY, UnitKind::Spearman, Hex::new(9, 8))
            .unit(21, ENEMY, UnitKind::Spearman, Hex::new(9, 9))
            .unit(22, ENEMY, UnitKind::Spearman, Hex::new(8, 9))
            .build();
        view.cities[0].is_capital = false;
        let d = view.cities[0].detail.as_mut().unwrap();
        d.captured = true;
        d.former_owner = Some(ENEMY);
        d.raze_blocker = None;
        let mut p = crate::fixture::planner(&view, 7);
        p.plan_raze();
        assert!(p.orders.contains(&Order::Raze {
            city: view.cities[0].id
        }));

        view.units.clear();
        let mut p = crate::fixture::planner(&view, 7);
        p.plan_raze();
        assert!(!p.orders.iter().any(|o| matches!(o, Order::Raze { .. })));
    }
}

#[cfg(test)]
mod purchase_savings_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ME, ViewBuilder};
    use legatum_core::{CityId, TechId};

    fn tally() -> Tally {
        Tally {
            army: 0,
            rangers: 0,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 0,
        }
    }

    fn fixture(gold: i32) -> legatum_core::PlayerView {
        let mut view = ViewBuilder::new(32, 20)
            .faction(Faction::Athens)
            .known_techs(&TechId::ALL)
            .gold(gold)
            .city(1, ME, Hex::from_offset(5, 5), 5)
            .city(2, ME, Hex::from_offset(12, 5), 5)
            .city(3, ME, Hex::from_offset(19, 5), 5)
            .build();
        view.economy.gold_per_turn.total = 50;
        for c in &mut view.cities[..2] {
            let d = c.detail.as_mut().unwrap();
            d.buildings.push(BuildingKind::Library);
            d.options.clear();
        }
        let d = view.cities[2].detail.as_mut().unwrap();
        let lab = ProductionItem::Building {
            building: BuildingKind::Library,
        };
        let granary = ProductionItem::Building {
            building: BuildingKind::Granary,
        };
        d.options.retain(|o| o.item == lab || o.item == granary);
        for o in &mut d.options {
            o.turns = 10;
            if o.item == lab {
                o.buy_cost = Some(250);
                o.buy_block =
                    (gold < 250).then_some(legatum_core::economy::BuyBlock::NotEnoughGold);
            } else {
                o.buy_cost = Some(80);
                o.buy_block = None;
            }
        }
        view
    }

    #[test]
    fn saves_for_third_lab_then_buys_it_at_the_quote() {
        let view = fixture(200);
        let mut baseline = Planner::new(
            &view,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        baseline.plan_buy(&mut tally());
        assert!(baseline.orders.contains(&Order::BuyItem {
            city: CityId(3),
            item: ProductionItem::Building {
                building: BuildingKind::Granary
            },
        }));

        let mut saving = Planner::new(
            &view,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        saving.decisions.purchase_savings_turns = 3;
        saving.plan_buy(&mut tally());
        assert!(saving.purchase_orders().is_empty());
        assert_eq!(saving.gold_left, 200);

        let funded = fixture(250);
        let mut saving = Planner::new(
            &funded,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        saving.decisions.purchase_savings_turns = 3;
        saving.plan_buy(&mut tally());
        assert!(saving.orders.contains(&Order::BuyItem {
            city: CityId(3),
            item: ProductionItem::Building {
                building: BuildingKind::Library
            },
        }));
    }
}

#[cfg(test)]
mod buy_hold_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ME, ViewBuilder};
    use legatum_core::{CityId, TechId};

    fn tally() -> Tally {
        Tally {
            army: 0,
            rangers: 0,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 0,
        }
    }

    /// The technology a Spearman's upgrade needs.
    fn phalangite_tech() -> TechId {
        data::unit_stats(UnitKind::Phalangite)
            .requires
            .expect("Phalangite tech")
    }

    /// Athens researching the Phalangite's technology (due in 5 turns) and knowing every
    /// other, four Spearmen, 50 Gold a turn, and one city offering only a Granary at 80 Gold.
    fn fixture(gold: i32) -> legatum_core::PlayerView {
        let known: Vec<_> = TechId::ALL
            .into_iter()
            .filter(|t| *t != phalangite_tech())
            .collect();
        let mut b = ViewBuilder::new(32, 20)
            .faction(Faction::Athens)
            .known_techs(&known)
            .researching(phalangite_tech())
            .gold(gold)
            .city(1, ME, Hex::from_offset(5, 5), 5)
            .city(2, ME, Hex::from_offset(12, 5), 5)
            .city(3, ME, Hex::from_offset(19, 5), 5);
        for id in 0..4 {
            b = b.unit(10 + id, ME, UnitKind::Spearman, Hex::from_offset(12, 5));
        }
        let mut view = b.build();
        view.research.turns_left = Some(5);
        view.economy.gold_per_turn.total = 50;
        for c in &mut view.cities[1..] {
            let d = c.detail.as_mut().unwrap();
            d.buildings.push(BuildingKind::Granary);
            d.options.clear();
        }
        let d = view.cities[0].detail.as_mut().unwrap();
        let granary = ProductionItem::Building {
            building: BuildingKind::Granary,
        };
        d.options.retain(|o| o.item == granary);
        for o in &mut d.options {
            o.turns = 10;
            o.buy_cost = Some(80);
            o.buy_block = None;
        }
        view
    }

    fn buys_granary(view: &legatum_core::PlayerView, gene: i32) -> bool {
        let mut p = Planner::new(
            view,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        p.decisions.buy_hold = gene;
        p.decisions.named_fund = 0;
        p.plan_buy(&mut tally());
        p.orders.contains(&Order::BuyItem {
            city: CityId(1),
            item: ProductionItem::Building {
                building: BuildingKind::Granary,
            },
        })
    }

    #[test]
    fn a_routine_buy_waits_while_the_gold_is_wanted_for_the_next_upgrades() {
        let probe = fixture(0);
        let reserve = Planner::new(
            &probe,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        )
        .gold_reserve();
        let hold = 25 * probe.economy.gold_per_turn.total;
        // The researched line is long enough to hit the income cap on reserved upgrade Gold.
        let gold = reserve + 80 + hold / 2;
        let view = fixture(gold);
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        p.decisions.buy_hold = 0;
        p.decisions.named_fund = 0;
        assert_eq!(p.held_gold(), 0, "gene off holds nothing");
        let mut held = Planner::new(
            &view,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        held.decisions.buy_hold = 1;
        held.decisions.named_fund = 0;
        assert_eq!(held.held_gold(), hold);
        assert!(buys_granary(&view, 0));
        assert!(!buys_granary(&view, 1));
        // Gold above the hold is spent as before; research further off holds nothing.
        assert!(buys_granary(&fixture(gold + hold), 1));
        let mut later = fixture(gold);
        later.research.turns_left = Some(20);
        assert!(buys_granary(&later, 1));
    }

    #[test]
    fn the_hold_is_capped_by_income() {
        let mut view = fixture(300);
        view.economy.gold_per_turn.total = 2;
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        p.decisions.buy_hold = 1;
        p.decisions.named_fund = 0;
        assert_eq!(p.held_gold(), 2 * 25);
    }

    #[test]
    fn hard_named_fund_upgrade_wave_escapes_the_income_cap() {
        let mut view = fixture(300);
        view.economy.gold_per_turn.total = 2;
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p.decisions.named_fund = 1;
        let wave = p.named_fund().expect("research due in five turns");
        assert!(
            matches!(wave.kind, crate::gold_hold::FundKind::Upgrade(t) if t == phalangite_tech())
        );
        assert!(!wave.ready);
        assert_eq!(p.held_gold(), wave.cost);
        assert!(wave.cost > 2 * 25, "the named bill is not capped by income");
        p.plan_buy(&mut tally());
        assert!(!p.orders.iter().any(|o| matches!(
            o,
            Order::BuyItem {
                item: ProductionItem::Building {
                    building: BuildingKind::Granary
                },
                ..
            }
        )));
    }

    #[test]
    fn hard_named_fund_spends_the_upgrade_wave_when_tech_lands() {
        let mut view = fixture(1000);
        view.research.known.push(phalangite_tech());
        for u in &mut view.units {
            if u.owner == ME {
                u.upgrade_to = Some(UnitKind::Phalangite);
                u.upgrade_cost = Some(100);
                u.upgrade_breakdown = Some(legatum_core::Breakdown {
                    total: 100,
                    parts: vec![],
                });
            }
        }
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p.decisions.named_fund = 1;
        let fund = p.named_fund().expect("known upgrade wave");
        assert!(
            matches!(fund.kind, crate::gold_hold::FundKind::Upgrade(t) if t == phalangite_tech())
        );
        assert!(fund.ready);
        assert_eq!(fund.cost, 400);
        p.plan_modernization();
        assert_eq!(
            p.orders
                .iter()
                .filter(|o| matches!(o, Order::UpgradeUnit { .. }))
                .count(),
            4
        );
    }

    #[test]
    fn hard_named_fund_emergency_waits_for_visible_army_to_reach_three_tiles() {
        let make = |enemy, gold| {
            let mut view = ViewBuilder::new(24, 16)
                .faction(Faction::Athens)
                .known_techs(&TechId::ALL)
                .gold(gold)
                .city(1, ME, Hex::new(6, 6), 5)
                .unit(2, crate::fixture::ENEMY, UnitKind::Spearman, enemy)
                .build();
            for option in &mut view.cities[0].detail.as_mut().unwrap().options {
                if matches!(option.item, ProductionItem::Unit { .. }) {
                    option.buy_cost = Some(2 * option.cost);
                }
            }
            view
        };
        let far = make(Hex::new(11, 6), 1300);
        let mut p = Planner::new(
            &far,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p.decisions.named_fund = 1;
        let fund = p.named_fund().expect("visible approach");
        assert_eq!(fund.kind, crate::gold_hold::FundKind::Emergency);
        assert!(!fund.ready);
        assert_eq!(p.held_gold(), fund.cost);
        let near = make(Hex::new(9, 6), 100);
        let mut p = Planner::new(
            &near,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p.decisions.named_fund = 1;
        let fund = p
            .named_fund()
            .expect("keep saving when the attacker reaches the door");
        assert!(fund.ready);
        assert!(fund.cost > 100);
    }

    #[test]
    fn hard_named_fund_buys_two_era_defenders_in_one_turn() {
        let mut view = ViewBuilder::new(24, 16)
            .faction(Faction::Athens)
            .known_techs(&TechId::ALL)
            .gold(5000)
            .city(1, ME, Hex::new(6, 6), 5)
            .city(2, ME, Hex::new(8, 6), 5)
            .unit(4, ME, UnitKind::Spearman, Hex::new(14, 6))
            .unit(
                3,
                crate::fixture::ENEMY,
                UnitKind::Spearman,
                Hex::new(11, 6),
            )
            .build();
        view.units
            .iter_mut()
            .find(|u| u.id == legatum_core::UnitId(4))
            .unwrap()
            .siege_offer = Some(legatum_core::view::SiegeOffer {
            price: legatum_core::Breakdown {
                total: 100,
                parts: vec![],
            },
            blocker: None,
            waits_for_gold: false,
        });
        for city in &mut view.cities {
            for option in &mut city.detail.as_mut().unwrap().options {
                if matches!(option.item, ProductionItem::Unit { .. }) {
                    option.buy_cost = Some(2 * option.cost);
                }
            }
        }
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p.decisions.named_fund = 1;
        let fund = p.named_fund().expect("two defenders for the visible front");
        assert_eq!(fund.kind, crate::gold_hold::FundKind::Emergency);
        assert!(fund.ready);
        p.siege_savings = 4800;
        p.orders.push(Order::UpgradeSiege {
            unit: legatum_core::UnitId(4),
        });
        p.gold_left -= 100;
        assert!(p.gold_left - fund.cost < p.gold_reserve());
        assert!(!p.buy_emergency_defender());
        assert!(
            !p.orders
                .iter()
                .any(|o| matches!(o, Order::UpgradeSiege { .. }))
        );
        let purchases: Vec<_> = p
            .orders
            .iter()
            .filter_map(|o| match o {
                Order::BuyItem {
                    city,
                    item: ProductionItem::Unit { unit },
                } => Some((*city, *unit)),
                _ => None,
            })
            .collect();
        assert_eq!(purchases.len(), 2);
        assert_ne!(purchases[0].0, purchases[1].0);
        assert!(
            purchases
                .iter()
                .all(|(_, unit)| data::unit_era_tier(*unit) >= 3)
        );
    }
}

#[cfg(test)]
mod modernization_tests {
    use super::*;
    #[test]
    fn modernization_reconsiders_hiroshimas_twenty_turn_archer_queue() {
        let view = crate::modernization::tests::saved_on(117, 5);
        let city = view
            .cities
            .iter()
            .find(|c| c.id == legatum_core::CityId(24))
            .unwrap();
        let archer = ProductionItem::Unit {
            unit: UnitKind::Archer,
        };
        assert_eq!(city.detail.as_ref().unwrap().producing, Some(archer));
        for enabled in [0, 100] {
            let mut p = crate::fixture::planner(&view, 1);
            p.decisions.modernization_value = enabled;
            p.decisions.modernization_upgrade = 0;
            p.decisions.modernization_return = 0;
            p.plan_cities();
            let chosen = p
                .orders
                .iter()
                .find_map(|o| match o {
                    Order::SetProduction { city: id, item } if *id == city.id => Some(*item),
                    _ => None,
                })
                .unwrap_or(archer);
            if enabled == 0 {
                assert_eq!(chosen, archer);
            } else {
                assert_ne!(chosen, archer, "{:?}", p.orders);
            }
        }
    }

    #[test]
    fn modernization_values_a_buildable_rifleman_above_obsolete_spearman() {
        let view = crate::modernization::tests::saved(2);
        let mut p = crate::fixture::planner(&view, 1);
        p.decisions.modernization_value = 100;
        let c = p.ctx.own_cities[0];
        let tally = Tally {
            army: 0,
            rangers: 0,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 0,
        };
        let options = &c.detail.as_ref().unwrap().options;
        let quote = |unit| {
            options
                .iter()
                .find(|o| o.item == ProductionItem::Unit { unit })
                .unwrap()
        };
        assert!(
            p.item_value(0, c, quote(UnitKind::Rifleman), &tally)
                > p.item_value(0, c, quote(UnitKind::Spearman), &tally)
        );
    }
}

#[cfg(test)]
#[path = "frontline_tests.rs"]
mod frontline_tests;

#[cfg(test)]
mod aztecs_tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off, planner};
    use crate::{Brain, ScriptedBrain};
    use legatum_core::AiDifficulty;

    #[test]
    fn aztecs_build_a_foot_defender_without_horses() {
        let v = ViewBuilder::new(24, 18)
            .faction(Faction::Aztecs)
            .city_with_detail(1, ME, off(5, 5), 3)
            .city(2, ENEMY, off(16, 5), 3)
            .unit(10, ENEMY, UnitKind::Horseman, off(7, 5))
            .build();
        let mut brain = ScriptedBrain::with_difficulty(Faction::Aztecs, 42, ME, AiDifficulty::Hard);
        let orders = brain.orders(&v);
        assert!(
            orders.iter().any(|o| matches!(o,
            Order::SetProduction { item: ProductionItem::Unit { unit }, .. }
            | Order::BuyItem { item: ProductionItem::Unit { unit }, .. }
            if composition::is_melee_line(*unit) || composition::is_ranged_line(*unit))),
            "{orders:?}"
        );
        assert!(!orders.iter().any(|o| matches!(o,
            Order::SetProduction { item: ProductionItem::Unit { unit }, .. }
            | Order::BuyItem { item: ProductionItem::Unit { unit }, .. }
            if data::is_horse_mounted(*unit))));
    }

    #[test]
    fn the_aztec_playbook_walls_a_frontier_city_first() {
        let walls = ProductionItem::Building {
            building: BuildingKind::Walls,
        };
        let view = |enemy_at| {
            ViewBuilder::new(30, 18)
                .faction(Faction::Aztecs)
                .known_techs(&[legatum_core::TechId::Masonry])
                .city_with_detail(1, ME, off(5, 5), 2)
                .city(2, ENEMY, enemy_at, 3)
                .build()
        };
        let mut params = crate::AiParams::builtin();
        params.decisions.aztecs_playbook = AZTEC_WALLS;
        // The plain Hard path (Hard Aztecs play the playbook by default).
        // A built-in faction entry wins over a shared value equal to the default, so the
        // gene goes off through the faction override.
        let mut stock = crate::AiParams::builtin();
        stock.faction_overrides.insert(
            Faction::Aztecs,
            crate::params::DecisionOverrides {
                aztecs_playbook: Some(0),
                ..Default::default()
            },
        );
        let walls_ordered = |v: &legatum_core::PlayerView, params: Option<&crate::AiParams>| {
            let mut brain = match params {
                Some(p) => {
                    ScriptedBrain::with_params(Faction::Aztecs, 42, ME, AiDifficulty::Hard, p)
                }
                None => ScriptedBrain::with_difficulty(Faction::Aztecs, 42, ME, AiDifficulty::Hard),
            };
            brain
                .orders(v)
                .iter()
                .any(|o| matches!(o, Order::SetProduction { item, .. } if *item == walls))
        };
        // A rival capital seven hexes off: the playbook walls first, the stock AI does not.
        let frontier = view(off(12, 5));
        assert!(walls_ordered(&frontier, Some(&params)));
        assert!(!walls_ordered(&frontier, Some(&stock)));
        // Twenty hexes off it is not a frontier.
        assert!(!walls_ordered(&view(off(25, 5)), Some(&params)));
    }

    #[test]
    fn the_aztec_playbook_puts_walls_before_the_shooter_unless_a_seen_army_comes_first() {
        let v = ViewBuilder::new(30, 18)
            .faction(Faction::Aztecs)
            .known_techs(&[legatum_core::TechId::Masonry])
            .city_with_detail(1, ME, off(5, 5), 2)
            .city(2, ENEMY, off(10, 5), 3)
            .build();
        let city = v.cities.iter().find(|c| c.owner == ME).unwrap();
        let mut p = planner(&v, 7);
        p.difficulty = AiDifficulty::Hard;
        // Stock: the rival city five hexes off is a deadline the Walls can't meet.
        assert!(!p.walls_beat_arrival(city));
        p.decisions.aztecs_playbook = AZTEC_WALLS;
        assert!(p.walls_beat_arrival(city), "no army in sight: Walls first");
        // A rival army in sight two hexes off still comes first.
        let seen = ViewBuilder { view: v.clone() }
            .unit(9, ENEMY, UnitKind::Spearman, off(7, 5))
            .build();
        let city = seen.cities.iter().find(|c| c.owner == ME).unwrap();
        let mut p = planner(&seen, 7);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.aztecs_playbook = AZTEC_WALLS;
        assert!(!p.walls_beat_arrival(city));
    }

    #[test]
    fn the_aztec_playbook_fights_a_war_with_a_lean_army() {
        use crate::plan::{Plan, PlanKind};
        let v = ViewBuilder::new(30, 18)
            .faction(Faction::Aztecs)
            .city_with_detail(1, ME, off(5, 5), 4)
            .city_with_detail(2, ME, off(5, 10), 3)
            .city(3, ENEMY, off(15, 5), 3)
            .build();
        let enemy = v.team_of(ENEMY);
        assert!(v.teams_at_war(v.team, enemy));
        let quota = |bits: i32| {
            let mut p = planner(&v, 7);
            p.difficulty = AiDifficulty::Hard;
            p.plan_routing = true;
            p.decisions.aztecs_playbook = bits;
            // The former shared war quota, so the Aztec cap is visible.
            p.decisions.army_war_extra = 5;
            p.campaigns.plan.active = Some(Plan {
                kind: PlanKind::Contain { against: enemy },
                start_turn: 1,
                review_turn: 10,
                reserve: 0,
                value: 300,
                terms: Vec::new(),
                breach: false,
                misses: 0,
            });
            assert!(p.contain_war().is_some());
            p.uncapped_army()
        };
        // Two cities: one guard each plus `army_war_extra` (pinned at 5), or plus one.
        assert_eq!(quota(0), 2 + 5);
        assert_eq!(quota(AZTEC_LEAN_ARMY), 2 + AZTEC_WAR_EXTRA);
    }

    #[test]
    fn aztecs_value_melee_captives_and_rivals_value_their_counters() {
        let v = ViewBuilder::new(24, 18)
            .faction(Faction::Aztecs)
            .city_with_detail(1, ME, off(5, 5), 5)
            .city(2, ENEMY, off(13, 5), 3)
            .build();
        let p = planner(&v, 42);
        assert!(
            p.flower_wars_build_value(
                &v.cities[0],
                ProductionItem::Unit {
                    unit: UnitKind::Spearman
                }
            ) > 0
        );
        assert_eq!(
            p.flower_wars_build_value(
                &v.cities[0],
                ProductionItem::Unit {
                    unit: UnitKind::Archer
                }
            ),
            0
        );
        let mut rival = v.clone();
        rival
            .players
            .iter_mut()
            .find(|x| x.id == ME)
            .unwrap()
            .faction = Faction::Athens;
        rival
            .players
            .iter_mut()
            .find(|x| x.id == ENEMY)
            .unwrap()
            .faction = Faction::Aztecs;
        let p = planner(&rival, 42);
        assert!(
            p.flower_wars_build_value(
                &rival.cities[0],
                ProductionItem::Unit {
                    unit: UnitKind::Archer
                }
            ) > 0
        );
        assert!(
            p.flower_wars_build_value(
                &rival.cities[0],
                ProductionItem::Building {
                    building: BuildingKind::Walls
                }
            ) > 0
        );
    }
}

#[cfg(test)]
mod walls_garrison_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::TechId;

    #[test]
    fn shared_gene_prices_visible_defense_against_troops_and_campaigns() {
        for faction in [Faction::India, Faction::Athens, Faction::Switzerland] {
            let mut view = ViewBuilder::new(26, 18)
                .faction(faction)
                .known_techs(&[TechId::Masonry])
                .city(1, ME, off(5, 5), 3)
                .city(2, ENEMY, off(13, 5), 3)
                .unit(10, ENEMY, UnitKind::Spearman, off(9, 5))
                .build();
            view.diplomacy.legacy = false;
            let mut p = Planner::new(&view, Knobs::new(40, 35, 40, 70, 20), faction, 1);
            assert_eq!(p.walls_garrison_ratio(&view.cities[0]), 0);
            p.decisions.walls_garrison_value = 100;
            let defensive = p.walls_garrison_ratio(&view.cities[0]);
            assert!(defensive > 0, "{faction:?}: {defensive}");
            // At the higher price, Walls no longer replace this city's troop outright.
            assert_eq!(p.troop_garrison_cities(), 1);
            p.strategy = Some(crate::Strategy::Rush);
            let offensive = p.walls_garrison_ratio(&view.cities[0]);
            assert!(
                defensive > offensive,
                "{faction:?}: {defensive} > {offensive}"
            );
            drop(p);
            let walls = ProductionItem::Building {
                building: BuildingKind::Walls,
            };
            view.rule_experiments.walls_cost = Some(120);
            let quoted = view.rule_experiments.base_production_cost(faction, walls);
            let option = view.cities[0]
                .detail
                .as_mut()
                .unwrap()
                .options
                .iter_mut()
                .find(|o| o.item == walls)
                .unwrap();
            option.cost = quoted;
            let mut expensive = Planner::new(&view, Knobs::new(40, 35, 40, 70, 20), faction, 1);
            expensive.decisions.walls_garrison_value = 100;
            assert!(expensive.walls_garrison_ratio(&view.cities[0]) < defensive);
        }
    }
}

#[cfg(test)]
mod swiss_playbook_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ME, ViewBuilder, off};
    use legatum_core::TechId;

    fn tally() -> Tally {
        Tally {
            army: 2,
            rangers: 1,
            scouts: 0,
            heir_pipeline: 0,
            settler_pipeline: 0,
            slow_settlers: 0,
            relay_started: false,
            sites: 0,
        }
    }

    /// Gene `swiss_playbook` 1: a founded Swiss city's Redoubt is its garrison. The quiet colony
    /// prices its Redoubt as the troop it replaces, the garrison floor stops at the two free
    /// units, and the capital keeps its usual Walls price.
    #[test]
    fn a_founded_swiss_citys_redoubt_is_its_garrison() {
        let view = ViewBuilder::new(30, 20)
            .faction(Faction::Switzerland)
            .known_techs(&[TechId::Masonry])
            .city(1, ME, off(6, 6), 4)
            .city(2, ME, off(14, 6), 2)
            .city(3, ME, off(6, 14), 2)
            .build();
        let walls = ProductionItem::Building {
            building: BuildingKind::Walls,
        };
        let value = |gene: i32, ci: usize| {
            let mut p = Planner::new(
                &view,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                7,
            );
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.decisions.swiss_playbook = gene;
            // The Swiss gene alone, not the shared rule (`walls_garrison`).
            p.decisions.walls_garrison = 0;
            let c = &view.cities[ci];
            let opt = c
                .detail
                .as_ref()
                .unwrap()
                .options
                .iter()
                .find(|o| o.item == walls)
                .expect("Masonry known: the Redoubt is buildable");
            (
                p.item_base_value(ci, c, opt, &tally()),
                p.troop_garrison_cities(),
                p.uncapped_army(),
            )
        };
        let (colony_off, troops_off, army_off) = value(0, 1);
        let (colony_on, troops_on, army_on) = value(1, 1);
        let (capital_off, _, _) = value(0, 0);
        let (capital_on, _, _) = value(1, 0);
        assert!(colony_off < GARRISON_FLOOR_VALUE, "{colony_off}");
        assert!(colony_on >= GARRISON_FLOOR_VALUE, "{colony_on}");
        assert_eq!(capital_on, capital_off);
        assert_eq!((troops_off, troops_on), (3, 0));
        assert_eq!(army_off, 3);
        assert_eq!(army_on, legatum_core::data::FREE_MILITARY_UNITS);
    }

    /// Gene `walls_garrison`: the same rule for any faction's Walls, and only on Hard.
    #[test]
    fn walls_garrison_applies_the_redoubt_rule_to_any_faction() {
        let view = ViewBuilder::new(30, 20)
            .faction(Faction::India)
            .known_techs(&[TechId::Masonry])
            .city(1, ME, off(6, 6), 4)
            .city(2, ME, off(14, 6), 2)
            .city(3, ME, off(6, 14), 2)
            .build();
        let walls = ProductionItem::Building {
            building: BuildingKind::Walls,
        };
        let value = |gene: i32, difficulty: legatum_core::AiDifficulty| {
            let mut p = Planner::new(&view, Knobs::for_faction(Faction::India), Faction::India, 7);
            p.difficulty = difficulty;
            p.decisions.walls_garrison = gene;
            let c = &view.cities[1];
            let opt = c
                .detail
                .as_ref()
                .unwrap()
                .options
                .iter()
                .find(|o| o.item == walls)
                .expect("Masonry known: Walls are buildable");
            (
                p.item_base_value(1, c, opt, &tally()),
                p.troop_garrison_cities(),
            )
        };
        let hard = legatum_core::AiDifficulty::Hard;
        let (colony_off, troops_off) = value(0, hard);
        let (colony_on, troops_on) = value(1, hard);
        assert!(colony_off < GARRISON_FLOOR_VALUE, "{colony_off}");
        assert!(colony_on >= GARRISON_FLOOR_VALUE, "{colony_on}");
        assert_eq!((troops_off, troops_on), (3, 0));
        assert_eq!(value(1, legatum_core::AiDifficulty::Normal).1, 3);
    }
}

#[cfg(test)]
mod unit_concentration_tests {
    use super::*;
    use crate::fixture::{ME, ViewBuilder, off, planner};

    #[test]
    fn slow_city_defers_a_duplicate_until_the_fast_city_finishes() {
        let mut view = ViewBuilder::new(20, 12)
            .city(1, ME, off(4, 4), 4)
            .city(2, ME, off(12, 4), 4)
            .build();
        let soldier = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        for (city, turns) in view.cities.iter_mut().zip([8, 20]) {
            city.detail
                .as_mut()
                .unwrap()
                .options
                .iter_mut()
                .find(|o| o.item == soldier)
                .unwrap()
                .turns = turns;
        }
        let slow_option = view.cities[1]
            .detail
            .as_ref()
            .unwrap()
            .options
            .iter()
            .find(|o| o.item == soldier)
            .unwrap();
        let mut p = planner(&view, 1);
        p.difficulty = AiDifficulty::Hard;
        assert!(p.defer_slow_duplicate_unit(&view.cities[1], slow_option));
        p.decisions.unit_concentration_turns = 0;
        assert!(!p.defer_slow_duplicate_unit(&view.cities[1], slow_option));
        p.decisions.unit_concentration_turns = 20;
        assert!(!p.defer_slow_duplicate_unit(&view.cities[1], slow_option));
    }
}

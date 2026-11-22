//! City decisions: what idle cities produce, occasional gold purchases.

use crate::ai_v1_baseline::analysis::{attack_power, defense_power};
use crate::ai_v1_baseline::scripted::{Planner, tag};
use legatum_core::data::MIN_CITY_GAP;
use legatum_core::view::{CityView, ProductionOption};
use legatum_core::{BuildingKind, Faction, Hex, Order, ProductionItem, UnitKind};

/// Sites scoring below this are not worth a settler.
pub const MIN_SITE_SCORE: i32 = 60;
/// Sites farther than this from every own city are not counted as settling room.
pub const MAX_SITE_DISTANCE: i32 = 10;

const RELAY: ProductionItem = ProductionItem::Building {
    building: BuildingKind::RelayUplink,
};

/// Empire-wide counts, updated as idle cities pick items so later cities see earlier picks.
struct Tally {
    army: i32,
    rangers: i32,
    scouts: i32,
    /// Settlers alive plus settlers being produced.
    settler_pipeline: i32,
    relay_started: bool,
    /// Distinct good sites available (capped).
    sites: i32,
}

fn is_military_unit(item: ProductionItem) -> bool {
    matches!(
        item,
        ProductionItem::Unit {
            unit: UnitKind::Soldier | UnitKind::Ranger | UnitKind::Trooper
        }
    )
}

impl Planner<'_> {
    pub fn plan_cities(&mut self) {
        let ctx = &self.ctx;
        let mut tally = Tally {
            army: 0,
            rangers: 0,
            scouts: 0,
            settler_pipeline: 0,
            relay_started: false,
            sites: self.count_good_sites(4),
        };
        for u in &ctx.own_units {
            match u.kind {
                UnitKind::Settler => tally.settler_pipeline += 1,
                UnitKind::Scout => tally.scouts += 1,
                UnitKind::Ranger => {
                    tally.army += 1;
                    tally.rangers += 1;
                }
                _ => tally.army += 1,
            }
        }
        for c in &ctx.own_cities {
            let d = c.detail.as_ref().expect("own city detail");
            match d.producing {
                Some(ProductionItem::Unit {
                    unit: UnitKind::Settler,
                }) => tally.settler_pipeline += 1,
                Some(ProductionItem::Unit {
                    unit: UnitKind::Scout,
                }) => tally.scouts += 1,
                Some(item) if is_military_unit(item) => tally.army += 1,
                Some(item) if item == RELAY => tally.relay_started = true,
                _ => {}
            }
        }

        let cities: Vec<(usize, &CityView)> = ctx.own_cities.iter().copied().enumerate().collect();
        for (ci, c) in cities {
            let d = c.detail.as_ref().expect("own city detail");
            if d.producing.is_none()
                && d.resistance <= 0
                && let Some(item) = self.choose_item(ci, c, &tally)
            {
                self.orders.push(Order::SetProduction { city: c.id, item });
                match item {
                    ProductionItem::Unit {
                        unit: UnitKind::Settler,
                    } => tally.settler_pipeline += 1,
                    ProductionItem::Unit {
                        unit: UnitKind::Scout,
                    } => tally.scouts += 1,
                    ProductionItem::Unit {
                        unit: UnitKind::Ranger,
                    } => {
                        tally.army += 1;
                        tally.rangers += 1;
                    }
                    i if is_military_unit(i) => tally.army += 1,
                    i if i == RELAY => tally.relay_started = true,
                    _ => {}
                }
            }
        }
        self.plan_buy();
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
                let near = ctx
                    .own_cities
                    .iter()
                    .any(|c| c.pos.distance(h) <= MAX_SITE_DISTANCE);
                (s >= MIN_SITE_SCORE && near && !ctx.in_danger(h)).then_some((s, h))
            })
            .collect();
        sites.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut picked: Vec<Hex> = Vec::new();
        for (_, h) in sites {
            if picked.len() as i32 >= cap {
                break;
            }
            if picked.iter().all(|p| p.distance(h) > MIN_CITY_GAP) {
                picked.push(h);
            }
        }
        picked.len() as i32
    }

    fn desired_army(&self) -> i32 {
        let k = self.knobs;
        let cities = self.ctx.own_cities.len() as i32;
        // Aggressive empires keep a standing field army on top of their garrisons.
        1 + cities * (20 + k.military + k.aggression / 2) / 70 + k.aggression / 25
    }

    /// Own military power standing within 2 of a city.
    fn local_defense(&self, c: &CityView) -> i32 {
        self.ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout && u.pos.distance(c.pos) <= 2)
            .map(|u| defense_power(self.view(), u).max(attack_power(u)))
            .sum()
    }

    fn choose_item(&self, ci: usize, c: &CityView, tally: &Tally) -> Option<ProductionItem> {
        let d = c.detail.as_ref()?;
        let mut best: Option<(i32, ProductionItem)> = None;
        for (oi, opt) in d.options.iter().enumerate() {
            let value = self.item_value(ci, c, opt, tally);
            if value <= 0 {
                continue;
            }
            let score = value * 30 / (opt.turns.clamp(0, 200) + 30)
                + self.jitter(tag::PRODUCTION, (c.id.0 as u64) << 8 | oi as u64, 15);
            if best.is_none_or(|(s, _)| score > s) {
                best = Some((score, opt.item));
            }
        }
        best.map(|(_, item)| item)
    }

    fn item_value(&self, ci: usize, c: &CityView, opt: &ProductionOption, t: &Tally) -> i32 {
        let k = self.knobs;
        let ctx = &self.ctx;
        let d = c.detail.as_ref().expect("own city detail");
        let threat = ctx.threat[ci];
        let threatened = threat > 0;
        let undefended = threat > self.local_defense(c);
        let cities = ctx.own_cities.len() as i32;
        let solvent = self.view().economy.gold_per_turn.total > 0 || t.army < 2;
        let has = |b: BuildingKind| d.buildings.contains(&b);
        let early = self.view().turn < 40;
        match opt.item {
            ProductionItem::Unit { unit } => match unit {
                UnitKind::Marksman
                | UnitKind::AssaultTrooper
                | UnitKind::Horseman
                | UnitKind::Knight
                | UnitKind::Cavalry
                | UnitKind::Tank
                | UnitKind::AutonomousArmour
                | UnitKind::Rifleman
                | UnitKind::Infantry
                | UnitKind::HunterKiller
                | UnitKind::FieldCannon
                | UnitKind::Artillery
                | UnitKind::Drone
                | UnitKind::Fighter
                | UnitKind::Bomber
                | UnitKind::StealthFighter
                | UnitKind::StealthBomber
                | UnitKind::Phalangite
                | UnitKind::CompositeArcher
                | UnitKind::Cataphract
                | UnitKind::Arquebusier
                | UnitKind::Cuirassier
                | UnitKind::TrenchInfantry
                | UnitKind::MechanisedInfantry
                | UnitKind::MainBattleTank
                | UnitKind::RocketArtillery
                | UnitKind::JetFighter
                | UnitKind::JetBomber
                | UnitKind::Howitzer
                | UnitKind::Landship
                | UnitKind::Biplane
                | UnitKind::Heir => 0,
                UnitKind::Settler => {
                    let pop_ok =
                        c.population >= 2 || d.turns_to_grow.is_some_and(|g| g <= opt.turns);
                    let want = cities + t.settler_pipeline < k.target_cities();
                    if !threatened && pop_ok && want && t.sites > t.settler_pipeline {
                        // The rising settler cost is already in opt.turns; also discount it.
                        400 + k.expansion * 5 - (opt.cost - 80)
                    } else {
                        0
                    }
                }
                UnitKind::Scout => {
                    let explore = !ctx.frontier.is_empty() && self.view().turn < 80;
                    if t.scouts == 0 && explore && !threatened {
                        200 + if self.faction == Faction::Byzantium {
                            150
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                UnitKind::Soldier | UnitKind::Ranger | UnitKind::Trooper => {
                    let kind_bonus = match unit {
                        UnitKind::Trooper => 150,
                        // Rangers defend: favored under threat and as a third of the army.
                        UnitKind::Ranger if threatened => 100,
                        UnitKind::Ranger if t.rangers * 3 < t.army + 1 => 40,
                        UnitKind::Ranger => -40,
                        _ => 0,
                    };
                    if undefended {
                        1000 + kind_bonus
                    } else if t.army < self.desired_army() && solvent {
                        200 + k.military * 5 + kind_bonus
                    } else {
                        0
                    }
                }
            },
            ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
                if has(building) {
                    return 0;
                }
                match building {
                    BuildingKind::Hydroponics
                    | BuildingKind::MachineShop
                    | BuildingKind::WarShrine
                    | BuildingKind::InferenceCore
                    | BuildingKind::College
                    | BuildingKind::Clearinghouse
                    | BuildingKind::StockExchange
                    | BuildingKind::Supermarket
                    | BuildingKind::AirDefence
                    | BuildingKind::StarFort
                    | BuildingKind::Autoworks
                    | BuildingKind::Castle
                    | BuildingKind::CourierHall
                    | BuildingKind::ImperialDiwan
                    | BuildingKind::Roadworks
                    | BuildingKind::Pyramids
                    | BuildingKind::HagiaSophia
                    | BuildingKind::EiffelTower
                    | BuildingKind::IterReactor => 0,
                    BuildingKind::BroodHall => {
                        let india = if self.faction == Faction::India {
                            80
                        } else {
                            0
                        };
                        150 + k.expansion + if early { 100 } else { 0 } - india
                    }
                    BuildingKind::Granary => 100 + k.economy * 2 + k.expansion,
                    BuildingKind::Workshop => 100 + k.economy * 2 + k.military,
                    BuildingKind::Market => {
                        let toll = if self.faction == Faction::Switzerland {
                            60
                        } else {
                            0
                        };
                        80 + k.economy * 3 + toll
                    }
                    BuildingKind::Laboratory => 80 + k.science * 4,
                    BuildingKind::Walls => {
                        let border = ctx.enemy_cities.iter().any(|e| e.pos.distance(c.pos) <= 8);
                        if threatened {
                            800 + k.military * 2
                        } else if border {
                            100 + k.military * 2
                        } else {
                            20
                        }
                    }
                    BuildingKind::PlanetaryFoundry | BuildingKind::WorldExchange => 0,
                    BuildingKind::RelayUplink => {
                        let best_producer = ctx
                            .own_cities
                            .iter()
                            .max_by_key(|o| {
                                let p = o.detail.as_ref().map_or(0, |d| d.yields.total.production);
                                (p, std::cmp::Reverse(o.id))
                            })
                            .is_some_and(|o| o.id == c.id);
                        let safe = !ctx.threat.iter().any(|t| *t > 0);
                        if !t.relay_started && best_producer && safe {
                            900 + k.science * 4
                        } else {
                            0
                        }
                    }
                }
            }
            ProductionItem::Project { .. } | ProductionItem::Improvement { .. } => 0,
        }
    }

    /// Buy at most one item per turn, when gold is plentiful: defenders for threatened cities
    /// first, then settlers, then anything else. Never the megaproject.
    fn plan_buy(&mut self) {
        let reserve = self.gold_reserve();
        let mut best: Option<(i32, i32, legatum_core::CityId, i32)> = None;
        for (ci, c) in self.ctx.own_cities.iter().enumerate() {
            let d = c.detail.as_ref().expect("own city detail");
            let (Some(item), Some(cost)) = (d.producing, d.buy_cost) else {
                continue;
            };
            if item == RELAY || cost <= 0 {
                continue;
            }
            let threatened = self.ctx.threat[ci] > 0;
            let (priority, keep) = if threatened && is_military_unit(item) {
                (3, reserve / 2)
            } else if matches!(
                item,
                ProductionItem::Unit {
                    unit: UnitKind::Settler
                }
            ) {
                (2, reserve)
            } else {
                (1, reserve * 2)
            };
            if self.gold_left - cost < keep {
                continue;
            }
            let better = best.is_none_or(|(bp, bc, bid, _)| {
                (priority, -cost, std::cmp::Reverse(c.id)) > (bp, -bc, std::cmp::Reverse(bid))
            });
            if better {
                best = Some((priority, cost, c.id, cost));
            }
        }
        if let Some((_, _, city, cost)) = best {
            self.gold_left -= cost;
            self.orders.push(Order::BuyItem {
                city,
                item: self
                    .ctx
                    .own_cities
                    .iter()
                    .find(|c| c.id == city)
                    .unwrap()
                    .detail
                    .as_ref()
                    .unwrap()
                    .producing
                    .unwrap(),
            });
        }
    }
}

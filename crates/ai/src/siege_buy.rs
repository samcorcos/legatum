//! Siege in the AI (user decision 2026-10-22, replacing the Military Engineer; siege study
//! 2026-10-18, `docs/design/research/siege-rework-2026-10-18.md`). Walls halve every attack on a
//! city's HP unless the attacking unit bought Siege (`rules::siege`: Gold, 1 Iron, in own land).
//! The AI reads the rule, never a shortcut:
//! - forecasts treat a force unit with Siege as covered, and one it could buy Siege for now (own
//!   land, Gold and Iron to spare) as covered at its price (`add_siege_cover`, gene
//!   `siege_for_walls`);
//! - before the army leaves, `plan_siege_buys` buys Siege for the force near a walled war
//!   target, the units that stand at home first, as far as the treasury keeps its reserve;
//! - Industrial and later land units have Siege built in (`data::has_siege`, designer
//!   2026-10-28): they count as covered everywhere above and are never offered a purchase.

use crate::scripted::Planner;
use legatum_core::data;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{Order, PlayerView, UnitId, UnitKind};

/// A walled city within this many tiles of a unit makes that unit a candidate for Siege.
const SIEGE_RADIUS: i32 = 9;
/// Most Siege units an army wants at once.
const MAX_SIEGE: i32 = 6;
/// Turns of Gold income a forecast may count on for the Siege it would buy (the army saves for it).
const SIEGE_SAVING_TURNS: i32 = 6;

/// What a forecast counted on for cover.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CoverUse {
    /// Force units that already have Siege.
    pub real: i32,
    /// Force units the forecast assumed we would buy Siege for.
    pub planned: i32,
    /// What the planned purchases cost, in Gold.
    pub gold: i32,
}

impl<'a> Planner<'a> {
    /// Whether we know a siege technology at all: Siege can be bought.
    pub(crate) fn siege_useful(&self) -> bool {
        let known: std::collections::BTreeSet<_> =
            self.view().research.known.iter().copied().collect();
        data::siege_tier(&known) > 0
    }

    /// Units of ours that already have Siege, bought or built in (`data::has_siege`).
    fn own_siege(&self) -> Vec<&'a UnitView> {
        self.ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| data::has_siege(u.kind, u.siege))
            .collect()
    }

    /// Units we could buy Siege for now or once the treasury has the price: its offer lacks at most Gold.
    fn siege_buyable(&self) -> Vec<&'a UnitView> {
        self.ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.military && !data::has_siege(u.kind, u.siege))
            .filter(|u| {
                u.siege_offer
                    .as_ref()
                    .is_some_and(|o| o.blocker.is_none() || o.waits_for_gold)
            })
            .collect()
    }

    /// Gold a forecast may count on for Siege at the moment the army could equip.
    fn siege_budget(&self) -> i32 {
        let future = if self.decisions.siege_timeline == 0 {
            SIEGE_SAVING_TURNS * self.view().economy.gold_per_turn.total.max(0)
        } else {
            0
        };
        self.gold_left + future - self.ordinary_gold_reserve()
    }

    /// The Iron supply left for more Siege units, `i32::MAX` when Siege uses none (or the view
    /// lists no Iron: a unit's offer already says so when none can be spared).
    fn siege_iron_spare(&self) -> i32 {
        if !data::SIEGE_USES_IRON {
            return i32::MAX;
        }
        self.view()
            .supply
            .iter()
            .find(|s| s.resource == legatum_core::resources::Resource::Iron)
            .map_or(i32::MAX, |s| s.spare)
    }

    /// Whether Walls are a price this AI can pay rather than a reason to look elsewhere: it reads
    /// them through Siege (gene `siege_for_walls`) and has Siege units or can buy two.
    pub(crate) fn walls_are_a_price(&self) -> bool {
        self.hard_defense()
            && self.decisions.siege_for_walls > 0
            && self.siege_useful()
            && (!self.own_siege().is_empty() || {
                let mut gold = self.siege_budget();
                let mut iron = self.siege_iron_spare();
                self.siege_buyable()
                    .into_iter()
                    .filter(|u| {
                        let price = u.siege_offer.as_ref().map_or(i32::MAX, |o| o.price.total);
                        if price > gold || iron < 1 {
                            return false;
                        }
                        gold -= price;
                        iron -= 1;
                        true
                    })
                    .take(2)
                    .count()
                    == 2
            })
    }

    /// Units a force of `n` needs covered: all of them, up to `MAX_SIEGE`.
    fn siege_wanted(n: usize) -> usize {
        n.min(MAX_SIEGE as usize)
    }

    /// The units a forecast should cover first: the melee wing that takes the city, then the
    /// shooters, lowest id first.
    fn cover_order(force: &mut [(UnitId, bool)]) {
        force.sort_by_key(|(id, charges)| (!*charges, *id));
    }

    /// Give the force units of a siege forecast on a walled `city` the Siege they have, and, with
    /// `allow_planned`, the Siege we could buy for the units at home (marked in `sim`). Returns
    /// how many of each it used and what the planned ones cost.
    pub(crate) fn add_siege_cover(
        &self,
        sim: &mut PlayerView,
        city: &CityView,
        force: &[UnitId],
        allow_planned: bool,
    ) -> CoverUse {
        let mut used = CoverUse::default();
        if !city.walls || force.is_empty() || !self.siege_useful() {
            return used;
        }
        let mut units: Vec<(UnitId, bool)> = force
            .iter()
            .filter_map(|id| {
                let u = sim.units.iter().find(|u| u.id == *id)?;
                Some((u.id, crate::composition::charges(u.kind)))
            })
            .collect();
        Self::cover_order(&mut units);
        let wanted = Self::siege_wanted(units.len());
        let real = units
            .iter()
            .take(wanted)
            .filter(|(id, _)| {
                sim.units
                    .iter()
                    .any(|u| u.id == *id && data::has_siege(u.kind, u.siege))
            })
            .count();
        used.real = real as i32;
        if !allow_planned || self.decisions.siege_for_walls == 0 {
            return used;
        }
        // The Gold we may spend and the Iron we can spare, buying the cheapest units first.
        let mut gold = self.siege_budget();
        let mut iron = self.siege_iron_spare();
        let mut planned: Vec<(UnitId, i32)> = Vec::new();
        for (id, _) in units.iter().take(wanted) {
            let Some(u) = sim
                .units
                .iter()
                .find(|u| u.id == *id && !data::has_siege(u.kind, u.siege))
            else {
                continue;
            };
            let Some(offer) = u
                .siege_offer
                .as_ref()
                .filter(|o| o.blocker.is_none() || o.waits_for_gold)
            else {
                continue;
            };
            let price = offer.price.total;
            if price > gold || iron < 1 {
                continue;
            }
            gold -= price;
            iron -= 1;
            planned.push((*id, price));
        }
        // All or nothing: a force covered in part is not a plan. A forecast that counted on cover
        // it could not buy sent armies at walls it could not breach (siege study, human-seat
        // benchmark 8.4 Sparta map: partial cover took the seat from 69% to 23%).
        if real + planned.len() < wanted || real + planned.len() < 2 {
            return used;
        }
        for (id, price) in planned {
            if let Some(u) = sim.units.iter_mut().find(|u| u.id == id) {
                u.siege = true;
            }
            used.planned += 1;
            used.gold += price;
        }
        used
    }

    /// The walled enemy city the war marches on: the campaign's target, else the nearest walled
    /// city of the team our plan fights, nearest our army first.
    fn walled_war_target(&self) -> Option<&'a CityView> {
        let v = self.view();
        self.campaigns
            .active
            .as_ref()
            .and_then(|c| {
                self.ctx
                    .enemy_cities
                    .iter()
                    .copied()
                    .find(|e| e.id == c.target)
            })
            .filter(|c| c.walls)
            .or_else(|| {
                let team = self.routed_plan().and_then(|p| p.kind.war_target())?;
                if !v.teams_at_war(v.team, team) {
                    return None;
                }
                self.ctx
                    .enemy_cities
                    .iter()
                    .copied()
                    .filter(|c| c.walls && v.team_of(c.owner) == team)
                    .min_by_key(|c| {
                        (
                            self.ctx
                                .own_units
                                .iter()
                                .filter(|u| u.military)
                                .map(|u| u.pos.distance(c.pos))
                                .min()
                                .unwrap_or(99),
                            c.id,
                        )
                    })
            })
    }

    /// Gene `refit_before_siege`: the force's refits that bring Siege built in (`data::built_in_siege`)
    /// cover a unit and make it stronger, for about what Siege alone costs. Every force unit
    /// without Siege whose refit gives a Siege-built-in kind, orderable now or unlocked by the
    /// technology we finish within the gene's turns, is one; their Gold, cheapest first, as far
    /// as Gold in hand plus that many turns of income reaches, is kept from Siege purchases.
    /// Returns that Gold and the refitting units (all of them, reserved or not: Siege bought for
    /// a unit about to refit is wasted).
    fn wall_proof_refits(&self, force: &[&UnitView]) -> (i32, Vec<UnitId>) {
        let turns = self.decisions.refit_before_siege;
        if turns == 0 {
            return (0, Vec::new());
        }
        let v = self.view();
        let soon = v
            .research
            .current
            .filter(|_| v.research.turns_left.is_some_and(|t| t <= turns));
        let halved = v
            .research
            .known
            .contains(&legatum_core::TechId::MilitaryEngineering);
        let mut refits: Vec<(i32, UnitId)> = force
            .iter()
            .filter(|u| !data::has_siege(u.kind, u.siege) && u.contract_until.is_none())
            .filter_map(|u| {
                if let Some(to) = u.upgrade_to {
                    let cost = u.upgrade_breakdown.as_ref()?.total;
                    return (data::built_in_siege(to) && u.upgrade_supply_blocker.is_none())
                        .then_some((cost, u.id));
                }
                let next = data::unit_stats(u.kind).upgrades_to?;
                let requires = data::unit_stats(next).requires?;
                if !data::built_in_siege(next) || soon != Some(requires) {
                    return None;
                }
                let cost = legatum_core::rules::units::upgrade_cost_for(self.faction, u.kind)?;
                Some((if halved { cost / 2 } else { cost }, u.id))
            })
            .collect();
        refits.sort();
        let mut reach = self.gold_left + turns * v.economy.gold_per_turn.total.max(0)
            - self.ordinary_gold_reserve();
        let mut gold = 0;
        for (cost, _) in &refits {
            if *cost > reach {
                break;
            }
            reach -= cost;
            gold += cost;
        }
        (gold, refits.into_iter().map(|(_, id)| id).collect())
    }

    /// Buy Siege for the force near the walled city our war marches on (gene `siege_for_walls`):
    /// one per unit of it, charging units first, units that stand at home (Siege is bought in own
    /// land, and the bought unit rests a turn) before the army leaves, while the treasury keeps
    /// its reserve and Iron is spare. A bought unit holds its tile this turn.
    pub(crate) fn plan_siege_buys(&mut self) {
        if !self.hard_defense() || self.decisions.siege_for_walls == 0 || !self.siege_useful() {
            return;
        }
        let Some(target) = self.walled_war_target() else {
            return;
        };
        let near = |u: &&UnitView| {
            u.military
                && u.kind != UnitKind::Scout
                && u.pos.distance(target.pos) <= SIEGE_RADIUS + 5
        };
        let force: Vec<&UnitView> = self.ctx.own_units.iter().copied().filter(near).collect();
        if force.is_empty() {
            return;
        }
        let (refit_gold, refitting) = self.wall_proof_refits(&force);
        let wanted = Self::siege_wanted(force.len()) as i32;
        let mut have = force
            .iter()
            .filter(|u| data::has_siege(u.kind, u.siege))
            .count() as i32;
        let mut buyers: Vec<&UnitView> = force
            .into_iter()
            .filter(|u| !data::has_siege(u.kind, u.siege) && !refitting.contains(&u.id))
            .filter(|u| {
                u.siege_offer
                    .as_ref()
                    .is_some_and(|o| o.blocker.is_none() || o.waits_for_gold)
            })
            // Never strip a city of its last military unit to rest it a turn.
            .filter(|u| {
                !self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                    || self
                        .ctx
                        .own_units
                        .iter()
                        .filter(|o| o.military && o.pos == u.pos && o.id != u.id)
                        .count()
                        > 0
            })
            .collect();
        buyers.sort_by_key(|u| {
            (
                !crate::composition::charges(u.kind),
                u.pos.distance(target.pos),
                u.id,
            )
        });
        let keep = self.ordinary_gold_reserve();
        let mut iron = self.siege_iron_spare();
        // The war needs `wanted - have` Siege units: what the treasury cannot pay yet is saved for.
        self.siege_savings = 0;
        let mut saving = 0;
        for u in buyers {
            if have >= wanted || iron < 1 || self.has_acted(u.id) {
                continue;
            }
            let offer = u.siege_offer.as_ref().expect("offer");
            let cost = offer.price.total;
            if offer.blocker.is_some() && !offer.waits_for_gold {
                continue;
            }
            // Gene `refit_before_siege`: the refits' Gold stays in hand, and nothing is saved
            // for Siege while one waits (the savings would hold back the refit itself).
            if refit_gold > 0
                && (offer.blocker.is_some() || self.gold_left - cost < keep + refit_gold)
            {
                continue;
            }
            if offer.blocker.is_some()
                || self.gold_left - cost < keep
                || !self.affordable_beyond_runway(cost)
            {
                saving += cost;
                continue;
            }
            self.gold_left -= cost;
            iron -= 1;
            have += 1;
            self.mil_dest.insert(u.pos);
            self.unit_order(u.id, Order::UpgradeSiege { unit: u.id });
        }
        // Set aside what is still to pay, so other purchases leave it (never more than a few turns of
        // income away: `siege_budget` is what a forecast counts on).
        let reachable =
            self.gold_left + SIEGE_SAVING_TURNS * self.view().economy.gold_per_turn.total.max(0);
        if saving > 0 && saving <= reachable {
            self.siege_savings = saving;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::view::SiegeOffer;
    use legatum_core::{AiDifficulty, Breakdown, TechId};

    fn planner(view: &PlayerView) -> Planner<'_> {
        let mut p = Planner::new(
            view,
            Knobs::new(50, 50, 50, 50, 100),
            view.faction_of(ME),
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p
    }

    /// Six Spearmen of ours stand in our land three tiles from a walled enemy city that our war
    /// marches on; each could buy Siege for `price` Gold.
    fn staged(price: i32, gold: i32) -> PlayerView {
        let mut b = ViewBuilder::new(30, 16)
            .known_techs(&[TechId::Masonry, TechId::Mathematics])
            .gold(gold)
            .city(1, ME, off(3, 8), 3)
            .city(9, ENEMY, off(12, 8), 3)
            .walled(9)
            .unit(20, ENEMY, UnitKind::Soldier, off(12, 8));
        for i in 0..6 {
            b = b.unit(
                i + 1,
                ME,
                UnitKind::Soldier,
                off(6 + i as i32 / 3, 7 + i as i32 % 3),
            );
        }
        let mut view = b.build();
        for u in view.units.iter_mut().filter(|u| u.owner == ME) {
            u.siege_offer = Some(SiegeOffer {
                price: Breakdown {
                    total: price,
                    parts: vec![("Siege, by the unit's era".into(), price)],
                },
                blocker: None,
                waits_for_gold: false,
            });
        }
        view
    }

    #[test]
    fn a_forecast_counts_units_with_siege_and_prices_the_ones_we_could_buy_it_for() {
        let view = staged(160, 2000);
        let city = view.cities.iter().find(|c| c.id.0 == 9).unwrap().clone();
        let ids: Vec<UnitId> = (1..=6).map(UnitId).collect();
        let p = planner(&view);
        let mut sim = view.clone();
        let used = p.add_siege_cover(&mut sim, &city, &ids, true);
        assert_eq!((used.real, used.planned, used.gold), (0, 6, 6 * 160));
        assert!(sim.units.iter().filter(|u| u.owner == ME).all(|u| u.siege));
        // Cover that is not for sale is not a plan: the force stays halved.
        let mut poor = view.clone();
        poor.economy.gold = 300;
        let mut sim = poor.clone();
        let p = planner(&poor);
        let used = p.add_siege_cover(&mut sim, &city, &ids, true);
        assert_eq!(used, CoverUse::default());
        assert!(!sim.units.iter().any(|u| u.siege));
        // Units that have Siege count without buying anything.
        let mut owned = view.clone();
        for u in owned.units.iter_mut().filter(|u| u.owner == ME) {
            u.siege = true;
            u.siege_offer = None;
        }
        let p = planner(&owned);
        let mut sim = owned.clone();
        let used = p.add_siege_cover(&mut sim, &city, &ids, false);
        assert_eq!((used.real, used.planned, used.gold), (6, 0, 0));
    }

    #[test]
    fn future_income_cannot_equip_the_border_force_today() {
        let mut view = staged(30, 0);
        view.economy.gold_per_turn.total = 100;
        let city = view.cities.iter().find(|c| c.id.0 == 9).unwrap();
        let ids = [UnitId(1), UnitId(2)];
        let p = planner(&view);
        let mut sim = view.clone();
        assert_eq!(p.add_siege_cover(&mut sim, city, &ids, true).planned, 0);
        let mut old = planner(&view);
        old.decisions.siege_timeline = 0;
        let mut sim = view.clone();
        assert_eq!(old.add_siege_cover(&mut sim, city, &ids, true).planned, 2);
    }

    #[test]
    fn the_army_buys_siege_before_it_leaves_for_the_walled_city_it_marches_on() {
        let view = staged(160, 2000);
        let mut p = planner(&view);
        p.decisions.siege_for_walls = 1;
        p.campaigns.active = Some(crate::campaign::Campaign {
            target: legatum_core::CityId(9),
            start_turn: 1,
            committed: Vec::new(),
            deadline: None,
            expected_value: 0,
            target_was_unwalled: false,
        });
        p.plan_siege_buys();
        let bought: Vec<UnitId> = p
            .orders
            .iter()
            .filter_map(|o| match o {
                Order::UpgradeSiege { unit } => Some(*unit),
                _ => None,
            })
            .collect();
        assert_eq!(bought.len(), 6, "{:?}", p.orders);
        assert!(p.gold_left < 2000 - 5 * 160, "the Gold was spent");
    }

    #[test]
    fn no_siege_is_bought_without_the_gene_without_a_walled_war_or_past_the_reserve() {
        let view = staged(160, 2000);
        let mut off_gene = planner(&view);
        off_gene.decisions.siege_for_walls = 0;
        off_gene.plan_siege_buys();
        assert!(off_gene.orders.is_empty());
        let mut peace = view.clone();
        peace.cities.retain(|c| c.id.0 != 9);
        let mut p = planner(&peace);
        p.plan_siege_buys();
        assert!(p.orders.is_empty());
        let broke = staged(160, 100);
        let mut p = planner(&broke);
        p.plan_siege_buys();
        assert!(p.orders.is_empty(), "{:?}", p.orders);
    }

    fn siege_bought(p: &Planner<'_>) -> Vec<UnitId> {
        p.orders
            .iter()
            .filter_map(|o| match o {
                Order::UpgradeSiege { unit } => Some(*unit),
                _ => None,
            })
            .collect()
    }

    fn marching(p: &mut Planner<'_>, gene: i32) {
        p.decisions.siege_for_walls = 1;
        p.decisions.refit_before_siege = gene;
        p.campaigns.active = Some(crate::campaign::Campaign {
            target: legatum_core::CityId(9),
            start_turn: 1,
            committed: Vec::new(),
            deadline: None,
            expected_value: 0,
            target_was_unwalled: false,
        });
    }

    #[test]
    fn a_refit_that_brings_siege_keeps_its_gold_from_siege_purchases() {
        // Spearmen 4 and 5 (nearest the city, bought first) can refit to Riflemen (Siege built in) for 300 Gold each.
        let mut view = staged(160, 700);
        for u in view.units.iter_mut().filter(|u| matches!(u.id.0, 4 | 5)) {
            u.upgrade_to = Some(UnitKind::Rifleman);
            u.upgrade_breakdown = Some(Breakdown {
                total: 300,
                parts: vec![("Base upgrade Gold".into(), 300)],
            });
        }
        let mut off_gene = planner(&view);
        marching(&mut off_gene, 0);
        off_gene.plan_siege_buys();
        assert!(
            siege_bought(&off_gene).contains(&UnitId(4)),
            "{:?}",
            off_gene.orders
        );
        // The two refits take 600 of the 700 Gold: no Siege is bought, none for a unit about
        // to refit, and nothing is saved for Siege while the refits wait.
        let mut p = planner(&view);
        marching(&mut p, 1);
        p.plan_siege_buys();
        assert!(siege_bought(&p).is_empty(), "{:?}", p.orders);
        assert_eq!(p.siege_savings, 0);
        // A rich treasury still buys Siege beyond the refits, never for the refitting units.
        let mut rich = view.clone();
        rich.economy.gold = 2000;
        let mut p = planner(&rich);
        marching(&mut p, 1);
        p.plan_siege_buys();
        let bought = siege_bought(&p);
        assert!(!bought.is_empty());
        assert!(!bought.contains(&UnitId(4)) && !bought.contains(&UnitId(5)));
    }

    #[test]
    fn a_refit_unlocked_by_the_technology_we_finish_next_turn_counts() {
        // Arquebusier 1 refits to a Field Cannon once Cannonry completes, next turn.
        let mut view = staged(160, 400);
        view.units.iter_mut().find(|u| u.id.0 == 1).unwrap().kind = UnitKind::Arquebusier;
        view.research.current = Some(TechId::Cannonry);
        view.research.turns_left = Some(1);
        let cost = legatum_core::rules::units::upgrade_cost_for(
            view.faction_of(ME),
            UnitKind::Arquebusier,
        )
        .unwrap();
        let force: Vec<&UnitView> = view.units.iter().filter(|u| u.owner == ME).collect();
        let mut p = planner(&view);
        p.decisions.refit_before_siege = 1;
        assert_eq!(p.wall_proof_refits(&force), (cost, vec![UnitId(1)]));
        // Two turns away is past the gene's window.
        view.research.turns_left = Some(2);
        let force: Vec<&UnitView> = view.units.iter().filter(|u| u.owner == ME).collect();
        let mut p = planner(&view);
        p.decisions.refit_before_siege = 1;
        assert_eq!(p.wall_proof_refits(&force), (0, Vec::new()));
    }

    #[test]
    fn industrial_units_count_as_wall_proof_and_are_never_bought_siege() {
        // Six Riflemen (Siege built in, designer 2026-10-28): the core offers them nothing.
        let mut view = staged(160, 2000);
        for u in view.units.iter_mut().filter(|u| u.owner == ME) {
            u.kind = UnitKind::Rifleman;
            u.siege_offer = None;
        }
        let city = view.cities.iter().find(|c| c.id.0 == 9).unwrap().clone();
        let ids: Vec<UnitId> = (1..=6).map(UnitId).collect();
        let p = planner(&view);
        let mut sim = view.clone();
        let used = p.add_siege_cover(&mut sim, &city, &ids, true);
        assert_eq!((used.real, used.planned, used.gold), (6, 0, 0));
        assert!(
            !sim.units.iter().any(|u| u.siege),
            "nothing is marked bought"
        );
        assert!(p.walls_are_a_price());
        let mut p = planner(&view);
        p.decisions.siege_for_walls = 1;
        p.campaigns.active = Some(crate::campaign::Campaign {
            target: legatum_core::CityId(9),
            start_turn: 1,
            committed: Vec::new(),
            deadline: None,
            expected_value: 0,
            target_was_unwalled: false,
        });
        p.plan_siege_buys();
        assert!(p.orders.is_empty(), "{:?}", p.orders);
        assert_eq!(p.siege_savings, 0, "nothing to save for");
    }
}

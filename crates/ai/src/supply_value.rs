//! What strategic supply is worth (user, 2026-10-19: resources are a core reason for war;
//! 2026-10-22: each era is defined by its resource).
//!
//! A deposit is worth what its supply would put to work or keep working, in Production per turn
//! (the public economy metric the win-chance model already counts, so a deposit lands in the
//! same currency as everything else a plan weighs). The users that wait on a resource are: idle
//! users first, then cities ready for the era's key building that needs it (Market, Forge,
//! University, Bank, Factory, Research Lab: its yield, priced like the tile score), units
//! whose upgrade into the era's army needs it (the stronger unit's premium, spread over the
//! planning horizon), and aircraft whose strength or construction needs Oil. Spare supply goes
//! to the most valuable users, so more supply is worth the next ones in line. Only public facts
//! and the seat's own view: a deposit counts once its
//! revealing technology is known (the view hides the rest), and a rival's loss is read from its
//! public city sizes, era, visible units and aircraft, never its buildings.

use legatum_core::data;
use legatum_core::experiments::RuleExperiments;
use legatum_core::resources::{self, Resource};
use legatum_core::view::CityView;
use legatum_core::{BuildingKind, Faction, Hex, PlayerId, PlayerView, UnitKind, Yields};
use std::collections::BTreeMap;

/// The key buildings that use `resource` (`resources::building_need`).
fn tiers(resource: Resource) -> impl Iterator<Item = BuildingKind> {
    resources::SUPPLY_TIERS
        .into_iter()
        .filter(move |b| resources::building_need(*b) == Some(resource))
}

/// A yield in Production per turn, priced like the tile score (Food 4, Production 3, Gold 2,
/// Science 3, over Production's 3).
fn production_worth(y: Yields) -> i32 {
    ((4 * y.food + 3 * y.production + 2 * y.gold + 3 * y.science) / 3).max(1)
}

/// Production per turn one supplied building tier gives.
fn building_worth(rules: RuleExperiments, faction: Faction, building: BuildingKind) -> i32 {
    production_worth(rules.building_stats_for(faction, building).yields)
}

/// Production per turn one supplied building user of `resource` gives (its best key building;
/// 0 when no building uses it).
pub(crate) fn per_user_production(
    rules: RuleExperiments,
    faction: Faction,
    resource: Resource,
) -> i32 {
    tiers(resource)
        .map(|b| building_worth(rules, faction, b))
        .max()
        .unwrap_or(0)
}

/// The users waiting on `resource`'s supply, worth in Production per turn each, most valuable
/// first, and the supply spare. Building users are idle users and cities ready for a key
/// building (a city a technology away counts half); unit users are own units whose upgrade
/// needs it. None until the deposits are revealed.
struct Demand {
    /// Idle users and building-ready cities, each worth a building's Production.
    buildings: Vec<i32>,
    /// Units whose upgrade needs it, each worth its strength premium.
    units: Vec<i32>,
    /// Aircraft whose lost strength could be restored, or whose useful construction awaits Oil.
    aircraft: Vec<i32>,
    spare: i32,
}

impl Demand {
    fn wanted(&self) -> i32 {
        (self.buildings.len() + self.units.len() + self.aircraft.len()) as i32
    }

    /// What `extra` more supply puts to work: the next users in line after the spare.
    fn gain(&self, extra: i32) -> i32 {
        let mut all: Vec<i32> = self
            .buildings
            .iter()
            .chain(&self.units)
            .chain(&self.aircraft)
            .copied()
            .collect();
        all.sort_unstable_by(|a, b| b.cmp(a));
        all.into_iter()
            .skip(self.spare.max(0) as usize)
            .take(extra.max(0) as usize)
            .sum()
    }
}

fn demand(view: &PlayerView, resource: Resource) -> Option<Demand> {
    let supply = view.supply.iter().find(|s| s.resource == resource)?;
    if !supply.revealed {
        return None;
    }
    let faction = view.faction_of(view.me);
    let idle_aircraft: Vec<_> = if resource == Resource::Oil {
        view.aircraft
            .iter()
            .filter(|a| a.owner == view.me && a.unsupplied == Some(resource))
            .collect()
    } else {
        Vec::new()
    };
    let mut buildings = vec![
        per_user_production(view.rule_experiments, faction, resource).max(1);
        supply.idle.len().saturating_sub(idle_aircraft.len())
    ];
    for building in tiers(resource).filter(|building| {
        !view
            .research
            .known
            .contains(&legatum_core::TechId::FusionPower)
            || !matches!(
                building,
                BuildingKind::Autoworks | BuildingKind::InferenceCore
            )
    }) {
        let tech = data::building_stats(building).requires;
        let known = tech.is_none_or(|t| view.research.known.contains(&t));
        let soon = known
            || tech.is_some_and(|t| {
                view.research.queue.contains(&t)
                    || view.research.current == Some(t)
                    || view.era_of(view.me) + 1 >= legatum_core::eras::era_of_tech(t)
            });
        if !soon {
            continue;
        }
        let gate = data::building_min_population(building);
        let below = data::building_predecessor(building);
        let ready = view
            .cities
            .iter()
            .filter(|c| c.owner == view.me)
            .filter_map(|c| c.detail.as_ref().map(|d| (c, d)))
            .filter(|(c, d)| {
                c.population >= gate - i32::from(!known)
                    && below
                        .is_none_or(|b| d.buildings.iter().any(|x| data::building_satisfies(*x, b)))
                    && !d
                        .buildings
                        .iter()
                        .any(|x| data::building_satisfies(*x, building))
            })
            .count();
        let ready = if known { ready } else { ready / 2 };
        buildings.extend(std::iter::repeat_n(
            building_worth(view.rule_experiments, faction, building),
            ready,
        ));
    }
    let units = view
        .units
        .iter()
        .filter(|u| u.owner == view.me && u.upgrade_need == Some(resource))
        .filter_map(|u| {
            let to = data::unit_stats(u.kind).upgrades_to?;
            let tech = data::unit_stats(to).requires;
            tech.is_none_or(|t| view.research.known.contains(&t))
                .then(|| (strength_premium_milli(faction, u.kind, to) / 1000).max(1))
        })
        .collect();
    let mut aircraft: Vec<i32> = idle_aircraft
        .iter()
        .map(|a| {
            (data::unit_stats_for(faction, a.kind).cost * resources::UNSUPPLIED_STRENGTH_PERCENT
                / 100
                / 30)
                .max(1)
        })
        .collect();
    if resource == Resource::Oil && view.research.known.contains(&legatum_core::TechId::Flight) {
        let own_cities: Vec<_> = view.cities.iter().filter(|c| c.owner == view.me).collect();
        let based = view.aircraft.iter().filter(|a| a.owner == view.me).count();
        let building = own_cities
            .iter()
            .filter(|c| c.detail.as_ref().is_some_and(|d| {
                matches!(d.producing, Some(legatum_core::ProductionItem::Unit { unit }) if data::is_aircraft(unit))
            }))
            .count();
        let vacancies = own_cities.len().saturating_sub(based + building);
        let mut prospects: Vec<i32> = own_cities
            .into_iter()
            .filter(|c| c.detail.as_ref().is_some_and(|d| {
                !matches!(d.producing, Some(legatum_core::ProductionItem::Unit { unit }) if data::is_aircraft(unit))
            }))
            .map(|c| {
                [
                    UnitKind::Biplane,
                    UnitKind::Bomber,
                    UnitKind::JetBomber,
                    UnitKind::Fighter,
                    UnitKind::JetFighter,
                ]
                    .into_iter()
                    .map(|unit| crate::air::production_value_with_policy(view, c, unit, 0, &[]) / 300)
                    .max()
                    .unwrap_or(0)
            })
            .filter(|worth| *worth > 0)
            .collect();
        prospects.sort_unstable_by(|a, b| b.cmp(a));
        aircraft.extend(prospects.into_iter().take(vacancies));
    }
    Some(Demand {
        buildings,
        units,
        aircraft,
        spare: supply.spare.max(0),
    })
}

/// Value of the best non-air use displaced when an aircraft takes one of the spare Oil slots.
/// A second spare slot only has a price when at least two other users are waiting.
pub(crate) fn aircraft_oil_opportunity(view: &PlayerView, war_percent: i32) -> i32 {
    let Some(d) = demand(view, Resource::Oil) else {
        return 0;
    };
    if d.spare <= 0 {
        return 0;
    }
    let at_war = view
        .players
        .iter()
        .any(|p| p.alive && !view.same_team(view.me, p.id) && view.at_war(view.me, p.id));
    let mut alternatives = d.buildings;
    alternatives.extend(
        d.units
            .into_iter()
            .map(|value| value * if at_war { war_percent } else { 100 } / 100),
    );
    alternatives.sort_unstable_by(|a, b| b.cmp(a));
    alternatives.get(d.spare as usize - 1).copied().unwrap_or(0)
}

/// Users of `resource` this seat would put to work with more supply: idle ones, cities ready for
/// its key building and units waiting to upgrade, minus supply already spare.
pub(crate) fn unmet(view: &PlayerView, resource: Resource) -> i32 {
    demand(view, resource).map_or(0, |d| (d.wanted() - d.spare).max(0))
}

/// This seat is short where it counts: some user is idle, or its era's resource has users
/// waiting with nothing spare.
pub(crate) fn short(view: &PlayerView) -> bool {
    view.supply.iter().any(|s| !s.idle.is_empty())
        || resources::era_resource(view.era_of(view.me))
            .is_some_and(|resource| unmet(view, resource) > 0)
}

/// Every spare unit of `resource` is already wanted by an idle user or a city ready for its key
/// building: any other use (a unit) takes a building's place.
pub(crate) fn contested(view: &PlayerView, resource: Resource) -> bool {
    demand(view, resource).is_some_and(|d| {
        let wanted = d.buildings.len() as i32;
        wanted > 0 && wanted >= d.spare
    })
}

/// Production per turn, in thousandths, the extra strength of `to` over `from` is worth: the
/// premium share of the cheaper unit's cost, spread over the 30-turn planning horizon. The
/// stronger unit is worth it where fights decide things; the same supply in a key building pays
/// `per_user_production` every turn.
fn strength_premium_milli(faction: Faction, from: UnitKind, to: UnitKind) -> i32 {
    let (a, b) = (
        data::unit_stats_for(faction, from),
        data::unit_stats_for(faction, to),
    );
    (b.strength - a.strength).max(0) * a.cost * 1000 / (a.strength.max(1) * 30)
}

/// A unit (or an upgrade into it) that uses `resource` is worth the supply it takes from the
/// buildings waiting for it: its strength premium over the unit it replaces, counted
/// `war_percent` percent at war (strength decides wars, a building's yield does not), against
/// the Production per turn the supply would put to work. Always true when nothing waits.
pub(crate) fn unit_outbids_industry(
    view: &PlayerView,
    from: UnitKind,
    to: UnitKind,
    resource: Resource,
    war_percent: i32,
    keshig_priority: bool,
) -> bool {
    // A promoted Horseman is ready now; a Forge's future yield is not the army's Iron bid.
    if keshig_priority
        && view.faction_of(view.me) == legatum_core::Faction::Mongols
        && from == UnitKind::Horseman
        && to == UnitKind::Cataphract
        && resource == Resource::Iron
    {
        return true;
    }
    if war_percent == 0 || !contested(view, resource) {
        return true;
    }
    let faction = view.faction_of(view.me);
    let at_war = view
        .players
        .iter()
        .any(|p| p.alive && !view.same_team(view.me, p.id) && view.at_war(view.me, p.id));
    let premium =
        strength_premium_milli(faction, from, to) * if at_war { war_percent } else { 100 } / 100;
    premium >= per_user_production(view.rule_experiments, faction, resource) * 1000
}

/// Production per turn `extra` more supply of `resource` would put to work for this seat.
pub(crate) fn own_gain(view: &PlayerView, resource: Resource, extra: i32) -> i32 {
    demand(view, resource).map_or(0, |d| d.gain(extra))
}

/// Production-equivalent value exposed if a deposit is lost. Spare supply protects current
/// users, though the lost capacity can still have value for the waiting users in `own_gain`.
pub(crate) fn own_loss(view: &PlayerView, resource: Resource, lost: i32) -> i32 {
    let Some(supply) = view
        .supply
        .iter()
        .find(|s| s.resource == resource && s.revealed)
    else {
        return 0;
    };
    let exposed = (lost - supply.spare.max(0)).max(0);
    let current = exposed
        * per_user_production(view.rule_experiments, view.faction_of(view.me), resource).max(1);
    current.max(own_gain(view, resource, lost))
}

/// Production per turn a rival loses with `lost` supply of `resource`: its public cities big
/// enough for a key building once its era has the technology, and its visible units that use it,
/// capped by what it loses.
pub(crate) fn rival_loss(view: &PlayerView, owner: PlayerId, resource: Resource, lost: i32) -> i32 {
    let faction = view.faction_of(owner);
    let era = view.era_of(owner);
    let mut users: Vec<i32> = Vec::new();
    for building in tiers(resource) {
        let tech_era = data::building_stats(building).requires.map_or(
            legatum_core::eras::FIRST_ERA,
            legatum_core::eras::era_of_tech,
        );
        if era < tech_era {
            continue;
        }
        let gate = data::building_min_population(building);
        let n = view
            .cities
            .iter()
            .filter(|c| c.owner == owner && c.population >= gate)
            .count();
        users.extend(std::iter::repeat_n(
            building_worth(view.rule_experiments, faction, building),
            n,
        ));
    }
    users.extend(
        view.units
            .iter()
            .filter(|u| u.owner == owner && resources::unit_need(u.kind) == Some(resource))
            .map(|u| {
                // An unsupplied unit loses a quarter of its strength: that share of its cost
                // over the horizon.
                (data::unit_stats_for(faction, u.kind).cost
                    * resources::UNSUPPLIED_STRENGTH_PERCENT
                    / 100
                    / 30)
                    .max(1)
            }),
    );
    if resource == Resource::Oil {
        users.extend(view.aircraft.iter().filter(|a| a.owner == owner).map(|a| {
            (data::unit_stats_for(faction, a.kind).cost * resources::UNSUPPLIED_STRENGTH_PERCENT
                / 100
                / 30)
                .max(1)
        }));
    }
    users.sort_unstable_by(|a, b| b.cmp(a));
    // Half: a rival with deposits elsewhere may have supply to spare.
    users.into_iter().take(lost.max(0) as usize).sum::<i32>() / 2
}

/// The strategic deposits the viewer knows on a city's land: tiles its owner holds that lie
/// nearer this city than any other of the owner's cities, with the supply each gives.
pub(crate) fn city_deposits(view: &PlayerView, city: &CityView) -> Vec<(Resource, i32)> {
    let others: Vec<Hex> = view
        .cities
        .iter()
        .filter(|c| c.owner == city.owner && c.id != city.id)
        .map(|c| c.pos)
        .collect();
    city.pos
        .within_iter(data::CITY_WORK_RADIUS + 1)
        .filter_map(|h| view.tile(h).map(|t| (h, t)))
        .filter(|(h, t)| {
            t.owner == Some(city.owner)
                && others
                    .iter()
                    .all(|o| o.distance(*h) > city.pos.distance(*h))
        })
        .filter_map(|(_, t)| {
            let r = t.resource.filter(|r| resources::is_strategic(*r))?;
            let mined = t.improvement.is_some();
            Some((
                r,
                resources::deposit_supply(
                    view.faction_of(city.owner),
                    view.rule_experiments.faction_rules_off,
                    mined,
                ),
            ))
        })
        .collect()
}

/// A capture's supply in the economy currency: what the city's deposits would put to work for
/// us, and what their owner loses.
pub(crate) fn capture_value(view: &PlayerView, city: &CityView) -> (i32, i32) {
    let mut supply: BTreeMap<Resource, i32> = BTreeMap::new();
    for (r, n) in city_deposits(view, city) {
        *supply.entry(r).or_default() += n;
    }
    let off = view.rule_experiments.faction_rules_off;
    let multiplier = |faction| {
        i32::from(
            faction == Faction::Germany
                && !off.contains(legatum_core::experiments::FactionRule::GermanyDoubleSupply),
        ) + 1
    };
    let own_factor = multiplier(view.faction_of(view.me));
    let former_factor = multiplier(view.faction_of(city.owner));
    supply.into_iter().fold((0, 0), |(gain, loss), (r, n)| {
        (
            gain + own_gain(view, r, n / former_factor * own_factor),
            loss + rival_loss(view, city.owner, r, n),
        )
    })
}

/// Supply a new city at `pos` would claim from known, unowned deposits in its work radius,
/// as Production per turn put to work for us.
pub(crate) fn site_gain(view: &PlayerView, pos: Hex, claim_timeline: i32) -> i32 {
    let mut supply: BTreeMap<Resource, [i32; 3]> = BTreeMap::new();
    for h in pos.within_iter(data::CITY_WORK_RADIUS) {
        let Some(t) = view.tile(h) else { continue };
        if (claim_timeline == 0 || t.owner.is_none())
            && let Some(r) = t.resource.filter(|r| resources::is_strategic(*r))
        {
            let ring = if claim_timeline == 0 {
                0
            } else {
                pos.distance(h).min(2) as usize
            };
            supply.entry(r).or_default()[ring] += resources::deposit_supply(
                view.players
                    .iter()
                    .find(|p| p.id == view.me)
                    .expect("own player")
                    .faction,
                view.rule_experiments.faction_rules_off,
                false,
            );
        }
    }
    supply
        .into_iter()
        .map(|(r, rings)| {
            let immediate = own_gain(view, r, rings[0] + rings[1]);
            let eventual = own_gain(view, r, rings.iter().sum());
            immediate + (eventual - immediate) / 2
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder};
    use legatum_core::resources::SupplyView;
    use legatum_core::types::Breakdown;
    use legatum_core::{Hex, TechId, UnitKind};

    fn supply(resource: Resource, supplied: i32, used: i32) -> SupplyView {
        let mut s = Breakdown::default();
        s.add("deposit", supplied);
        let mut u = Breakdown::default();
        u.add("users", used);
        SupplyView {
            resource,
            revealed_by: None,
            revealed: true,
            supplied: s,
            deposits: Vec::new(),
            used: u,
            idle: Vec::new(),
            held: Vec::new(),
            spare: supplied - used,
        }
    }

    fn ready_view(cities: u32) -> PlayerView {
        let mut b = ViewBuilder::new(30, 20);
        for i in 0..cities {
            b = b.city_with_detail(i + 1, ME, Hex::new(3 + 6 * i as i32, 5), 5);
        }
        let mut v = b.build();
        for c in &mut v.cities {
            c.detail.as_mut().unwrap().buildings = vec![BuildingKind::Workshop];
        }
        v.research.known.push(TechId::IronWorking);
        v.research.known.push(TechId::BronzeWorking);
        v
    }

    #[test]
    fn a_settlement_only_gains_new_deposits_on_the_claim_timeline() {
        let mut v = ready_view(1);
        v.supply = vec![supply(Resource::Iron, 0, 0)];
        let site = Hex::new(14, 10);
        let owned = Hex::new(15, 10);
        let outer = Hex::new(16, 10);
        let i = v.index(owned).unwrap();
        v.tiles[i].owner = Some(ME);
        v.tiles[i].resource = Some(Resource::Iron);
        assert_eq!(site_gain(&v, site, 1), 0, "owned supply adds nothing");
        let i = v.index(outer).unwrap();
        v.tiles[i].resource = Some(Resource::Iron);
        assert!(
            site_gain(&v, site, 1) < own_gain(&v, Resource::Iron, 2),
            "outer-ring supply arrives after founding"
        );
        assert!(
            site_gain(&v, site, 0) > 0,
            "zero reproduces the old valuation"
        );
    }

    #[test]
    fn iron_is_worth_the_forges_it_would_put_to_work() {
        let mut v = ready_view(3);
        v.supply = vec![supply(Resource::Iron, 0, 0), supply(Resource::Coal, 0, 0)];
        let forge = building_worth(
            v.rule_experiments,
            v.faction_of(ME),
            BuildingKind::MachineShop,
        );
        assert_eq!(unmet(&v, Resource::Iron), 3);
        assert_eq!(own_gain(&v, Resource::Iron, 2), 2 * forge);
        // Spare supply already covers one city.
        v.supply[0] = supply(Resource::Iron, 2, 1);
        assert_eq!(unmet(&v, Resource::Iron), 2);
        // Enough supply for every ready city: a new deposit is worth nothing more.
        v.supply[0] = supply(Resource::Iron, 6, 0);
        assert_eq!(own_gain(&v, Resource::Iron, 3), 0);
        // An idle user counts first.
        v.supply[0] = supply(Resource::Iron, 2, 3);
        v.supply[0].idle = vec!["Forge in C1".into()];
        v.supply[0].spare = -1;
        assert_eq!(unmet(&v, Resource::Iron), 4);
    }

    #[test]
    fn a_pikeman_waits_for_a_forge_unless_a_war_makes_its_strength_pay() {
        let mut v = ready_view(2);
        let faction = v.faction_of(ME);
        // No war: the fixture's default is the legacy all-war rule.
        v.diplomacy.legacy = false;
        v.supply = vec![supply(Resource::Iron, 2, 0), supply(Resource::Coal, 0, 0)];
        let outbids = |v: &PlayerView, pct| {
            unit_outbids_industry(
                v,
                UnitKind::Phalangite,
                UnitKind::Trooper,
                Resource::Iron,
                pct,
                false,
            )
        };
        // Two spare Iron and two Forge-ready cities: every spare Iron is wanted.
        assert!(contested(&v, Resource::Iron));
        assert!(
            strength_premium_milli(faction, UnitKind::Phalangite, UnitKind::Trooper)
                < per_user_production(v.rule_experiments, faction, Resource::Iron) * 1000
        );
        assert!(!outbids(&v, 800));
        // Spare Iron beyond what the cities want: nothing is displaced.
        v.supply[0] = supply(Resource::Iron, 4, 0);
        assert!(!contested(&v, Resource::Iron));
        assert!(outbids(&v, 800));
        // At war the unit's strength counts several times over.
        v.supply[0] = supply(Resource::Iron, 2, 0);
        v.diplomacy.legacy = true;
        assert!(outbids(&v, 2000));
        assert!(!outbids(&v, 100));
    }

    #[test]
    fn a_hidden_resource_is_worth_nothing() {
        let mut v = ready_view(2);
        v.supply = vec![supply(Resource::Iron, 0, 0)];
        v.supply[0].revealed = false;
        assert_eq!(own_gain(&v, Resource::Iron, 2), 0);
    }

    #[test]
    fn oil_supply_prices_a_useful_bomber_even_without_a_ready_lab() {
        let mut v = ViewBuilder::new(24, 16)
            .city_with_detail(1, ME, Hex::new(5, 8), 5)
            .city(2, ENEMY, Hex::new(12, 8), 5)
            .build();
        v.research.known.push(TechId::Flight);
        v.supply = vec![supply(Resource::Oil, 0, 0)];
        assert_eq!(unmet(&v, Resource::Oil), 1);
        assert!(own_gain(&v, Resource::Oil, 1) > 0);
        // A second Oil source has no further air use in this one-city empire.
        assert_eq!(
            own_gain(&v, Resource::Oil, 2),
            own_gain(&v, Resource::Oil, 1)
        );
    }

    #[test]
    fn an_aircraft_prices_the_lab_it_would_displace() {
        let mut v = ready_view(1);
        v.cities[0].population = 8;
        v.cities[0].detail.as_mut().unwrap().buildings = vec![BuildingKind::College];
        v.research.known.push(TechId::Electronics);
        v.supply = vec![supply(Resource::Oil, 1, 0)];
        assert_eq!(
            aircraft_oil_opportunity(&v, 800),
            building_worth(
                v.rule_experiments,
                v.faction_of(ME),
                BuildingKind::InferenceCore,
            )
        );
        v.supply[0] = supply(Resource::Oil, 2, 0);
        assert_eq!(aircraft_oil_opportunity(&v, 800), 0);
    }

    #[test]
    fn spare_oil_absorbs_a_lost_deposit_before_users_are_exposed() {
        let mut v = ready_view(1);
        v.supply = vec![supply(Resource::Oil, 3, 0)];
        assert_eq!(own_loss(&v, Resource::Oil, 2), 0);
        v.supply[0] = supply(Resource::Oil, 1, 1);
        assert!(own_loss(&v, Resource::Oil, 2) > 0);
    }

    #[test]
    fn a_city_holding_iron_is_a_prize_and_its_owner_loses_by_its_size() {
        let mut v = ready_view(4);
        v.supply = vec![supply(Resource::Iron, 0, 0), supply(Resource::Coal, 0, 0)];
        let enemy = ViewBuilder::new(30, 20)
            .city(9, ENEMY, Hex::new(20, 12), 6)
            .build()
            .cities
            .remove(0);
        v.cities.push(enemy.clone());
        let deposit = Hex::new(21, 12);
        let i = v.index(deposit).unwrap();
        v.tiles[i].owner = Some(ENEMY);
        v.tiles[i].resource = Some(Resource::Iron);
        assert_eq!(city_deposits(&v, &enemy), vec![(Resource::Iron, 2)]);
        let (gain, _) = capture_value(&v, &enemy);
        assert_eq!(
            gain,
            2 * building_worth(
                v.rule_experiments,
                v.faction_of(ME),
                BuildingKind::MachineShop,
            )
        );
        v.players.iter_mut().find(|p| p.id == ME).unwrap().faction = Faction::Germany;
        let (german_gain, _) = capture_value(&v, &enemy);
        assert_eq!(
            german_gain,
            4 * building_worth(
                v.rule_experiments,
                Faction::Germany,
                BuildingKind::MachineShop
            )
        );
    }
}

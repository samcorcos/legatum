//! Map resources (G11) and strategic supply (user, 2026-10-19: resources are a core reason for war).
//!
//! Wheat, Cattle, Stone and Copper are *bonus* resources (always visible, a tile bonus once improved).
//! Strategic resources begin in the Classical era (`ERA_RESOURCES`): Iron (Classical and Medieval),
//! Niter, Coal, Oil (Machine Age, World War and Modern) and Silicon, each hidden until its owner knows its revealing technology.
//!
//! **Supply, not a stockpile.** Each known strategic deposit inside an empire's borders supplies
//! `SUPPLY_PER_DEPOSIT` of its resource, plus `SUPPLY_PER_MINE` with a Mine on it. An era's new
//! units and its key building use 1 of that era's resource for as long as they exist
//! (`unit_need`, `building_need`): the units from the Classical era on (turn-1 units are free; an
//! upgrade switches to the new era's resource), and Forge, University, Bank, Factory and
//! Research Lab (a higher tier keeps its lower tiers' needs). A new
//! user can start, be bought or be upgraded to only while supply is spare, counting builds in
//! progress that hold supply for when they finish. Supply is counted at the end of every turn
//! (`update_supply`): when an empire uses more than it has (a deposit lost, a city captured), the
//! newest users go idle until supply returns. An idle building yields nothing; an idle unit
//! fights with `UNSUPPLIED_STRENGTH_PERCENT` less strength.
//!
//! Placement (`placement`) is a separate, seeded mapgen layer run after starts are chosen.

use crate::data;
use crate::events::Event;
use crate::hex::Hex;
use crate::state::{City, GameState, Player, Tile, Unit};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub mod placement;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Resource {
    Wheat,
    Cattle,
    Stone,
    Copper,
    Iron,
    Coal,
    Niter,
    Oil,
    #[serde(alias = "Lithium")]
    Silicon,
}

impl Resource {
    pub const ALL: [Resource; 9] = [
        Resource::Wheat,
        Resource::Cattle,
        Resource::Stone,
        Resource::Copper,
        Resource::Iron,
        Resource::Niter,
        Resource::Coal,
        Resource::Oil,
        Resource::Silicon,
    ];
    pub const BONUS: [Resource; 4] = [
        Resource::Wheat,
        Resource::Cattle,
        Resource::Stone,
        Resource::Copper,
    ];
    /// In era order (`ERA_RESOURCES`).
    pub const STRATEGIC: [Resource; 5] = [
        Resource::Iron,
        Resource::Niter,
        Resource::Coal,
        Resource::Oil,
        Resource::Silicon,
    ];
}

/// Each era's resource, by 0-based era (`eras::current_era`): none in Ancient, Classical and
/// Medieval Iron, Renaissance Niter, Industrial Coal, Machine Age, World War and Modern Oil
/// (user, 2026-11-14: "it's ok if oil covers 2 eras"; the Modern era, added the same day as a
/// placeholder, makes it three), Future Silicon.
pub const ERA_RESOURCES: [Option<Resource>; crate::eras::ERA_COUNT] = [
    None,
    Some(Resource::Iron),
    Some(Resource::Iron),
    Some(Resource::Niter),
    Some(Resource::Coal),
    Some(Resource::Oil),
    Some(Resource::Oil),
    Some(Resource::Oil),
    Some(Resource::Silicon),
];

/// The resource of a 0-based era.
pub fn era_resource(era: u8) -> Option<Resource> {
    ERA_RESOURCES[usize::from(era).min(ERA_RESOURCES.len() - 1)]
}

/// The building tiers that use supply, lowest era first (`building_need`).
/// The Machine Age's Stock Exchange uses nothing: the era shares the World War era's Oil, and its units and
/// aircraft already draw on it (lead's call, 2026-11-14).
pub const SUPPLY_TIERS: [BuildingKind; 5] = [
    BuildingKind::MachineShop,
    BuildingKind::College,
    BuildingKind::Clearinghouse,
    BuildingKind::Autoworks,
    BuildingKind::InferenceCore,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResourceClass {
    Bonus,
    Strategic,
}

pub struct ResourceStats {
    pub class: ResourceClass,
    /// Terrain the resource is placed on (the guaranteed Iron may fall back to other land).
    pub terrains: &'static [Terrain],
    /// Added to the tile's yields when a citizen works it and its owner knows the resource.
    pub bonus: Yields,
    /// Strategic resources are hidden (no yield, no supply) until this is known.
    pub revealed_by: Option<TechId>,
}

pub fn stats(resource: Resource) -> ResourceStats {
    use Terrain::*;
    let (class, terrains, bonus, revealed_by): (_, &'static [Terrain], _, _) = match resource {
        Resource::Wheat => (
            ResourceClass::Bonus,
            &[Plains, Grassland],
            Yields::new(1, 0, 0, 0),
            None,
        ),
        Resource::Cattle => (
            ResourceClass::Bonus,
            &[Grassland],
            Yields::new(1, 0, 0, 0),
            None,
        ),
        Resource::Stone => (
            ResourceClass::Bonus,
            &[Grassland, Plains, Hills],
            Yields::new(0, 1, 0, 0),
            None,
        ),
        // Copper keeps its tile yield and remains visible from the start, but supplies nothing.
        Resource::Copper => (
            ResourceClass::Bonus,
            &[Hills],
            Yields::new(0, 0, 2, 0),
            None,
        ),
        Resource::Iron => (
            ResourceClass::Strategic,
            &[Hills, Tundra],
            Yields::new(0, 1, 0, 0),
            Some(TechId::IronWorking),
        ),
        Resource::Niter => (
            ResourceClass::Strategic,
            &[Plains, Desert, Grassland],
            Yields::new(1, 1, 0, 0),
            Some(TechId::Gunpowder),
        ),
        Resource::Coal => (
            ResourceClass::Strategic,
            &[Hills, Forest],
            Yields::new(0, 2, 0, 0),
            Some(TechId::Industrialization),
        ),
        Resource::Oil => (
            ResourceClass::Strategic,
            &[Desert, Tundra, Plains],
            Yields::new(0, 1, 1, 0),
            Some(TechId::Combustion),
        ),
        Resource::Silicon => (
            ResourceClass::Strategic,
            &[Desert, Hills],
            Yields::new(0, 1, 0, 1),
            Some(TechId::AutonomousSystems),
        ),
    };
    ResourceStats {
        class,
        terrains,
        bonus,
        revealed_by,
    }
}

pub fn name(resource: Resource) -> &'static str {
    match resource {
        Resource::Wheat => "Wheat",
        Resource::Cattle => "Cattle",
        Resource::Stone => "Stone",
        Resource::Copper => "Copper",
        Resource::Iron => "Iron",
        Resource::Coal => "Coal",
        Resource::Niter => "Niter",
        Resource::Oil => "Oil",
        Resource::Silicon => "Silicon",
    }
}

pub fn is_strategic(resource: Resource) -> bool {
    stats(resource).class == ResourceClass::Strategic
}

/// Supply of a known strategic deposit inside your borders.
pub const SUPPLY_PER_DEPOSIT: i32 = 2;
/// Extra supply from a Mine on the deposit (or a city built on it).
pub const SUPPLY_PER_MINE: i32 = 1;
/// An unsupplied unit loses this percent of its base strength in every fight.
pub const UNSUPPLIED_STRENGTH_PERCENT: i32 = 25;
/// Settle scoring: each known strategic deposit within the work radius is worth this much.
pub const STRATEGIC_SITE_VALUE: i32 = 12;
/// Border growth leans toward known strategic deposits, as Civ's does toward resources:
/// the deposit tile scores this much extra...
pub const BORDER_PULL_DEPOSIT: i32 = 3;
/// ...and an unowned tile next to an unclaimed known deposit in the city's reach this much.
pub const BORDER_PULL_NEXT: i32 = 2;

/// Whether an owner with these techs knows the resource (bonus resources are always known).
pub fn known(resource: Resource, knows: impl Fn(TechId) -> bool) -> bool {
    stats(resource).revealed_by.is_none_or(knows)
}

pub fn known_to(resource: Resource, techs: &BTreeSet<TechId>) -> bool {
    known(resource, |t| techs.contains(&t))
}

/// The resource part of a tile's yield for an owner with this knowledge.
pub fn tile_bonus(resource: Option<Resource>, knows: impl Fn(TechId) -> bool) -> Yields {
    match resource {
        Some(r) if known(r, knows) => stats(r).bonus,
        _ => Yields::ZERO,
    }
}

/// What a worked tile yields: terrain (faction-adjusted) plus its resource, which counts only
/// once the tile is improved (`improvements::tile_resource_yields`).
pub fn tile_yields(faction: Faction, tile: &Tile, knows: impl Fn(TechId) -> bool) -> Yields {
    if tile.landmark.is_some() {
        return Yields::ZERO;
    }
    data::terrain_yields(faction, tile.terrain)
        + crate::improvements::tile_resource_yields(tile, false, knows)
}

/// What a tile would yield with its known resource improved: the land's worth when choosing which
/// tiles to claim or keep, before anyone has built on it.
pub fn tile_potential_yields(
    faction: Faction,
    tile: &Tile,
    knows: impl Fn(TechId) -> bool,
) -> Yields {
    if tile.landmark.is_some() {
        return Yields::ZERO;
    }
    data::terrain_yields(faction, tile.terrain)
        + tile.resource.map_or(Yields::ZERO, |r| {
            crate::improvements::resource_yields(r, knows)
        })
}

/// Extra border-growth score for `h` claimed by a city at `city_pos` (see `BORDER_PULL_*`).
pub fn border_pull(state: &GameState, owner: PlayerId, city_pos: Hex, h: Hex) -> i32 {
    let techs = &state.player(owner).techs;
    let deposit = |x: Hex| {
        state.map.get(x).is_some_and(|t| {
            t.owner.is_none()
                && t.resource
                    .is_some_and(|r| is_strategic(r) && known_to(r, techs))
        })
    };
    if deposit(h) {
        BORDER_PULL_DEPOSIT
    } else if h
        .neighbors()
        .into_iter()
        .any(|n| n.distance(city_pos) <= crate::data::CITY_WORK_RADIUS && deposit(n))
    {
        BORDER_PULL_NEXT
    } else {
        0
    }
}

/// Strategic deposits this empire supplies from: known deposits on tiles it owns (worked or not).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deposit {
    pub resource: Resource,
    pub at: Hex,
    pub city: Option<CityId>,
    /// A Mine on the deposit (or a city built on it) adds `SUPPLY_PER_MINE`.
    pub mined: bool,
}

impl Deposit {
    pub fn supply(&self) -> i32 {
        SUPPLY_PER_DEPOSIT + if self.mined { SUPPLY_PER_MINE } else { 0 }
    }
}

/// Supply from one deposit after the owner's faction rules are applied.
pub fn deposit_supply(
    faction: Faction,
    off: crate::experiments::FactionRuleSet,
    mined: bool,
) -> i32 {
    let base = SUPPLY_PER_DEPOSIT + if mined { SUPPLY_PER_MINE } else { 0 };
    if faction == Faction::Germany
        && !off.contains(crate::experiments::FactionRule::GermanyDoubleSupply)
    {
        2 * base
    } else {
        base
    }
}

pub fn mine_supply_gain(faction: Faction, off: crate::experiments::FactionRuleSet) -> i32 {
    deposit_supply(faction, off, true) - deposit_supply(faction, off, false)
}

pub fn deposits(state: &GameState, p: PlayerId) -> Vec<Deposit> {
    let player = state.player(p);
    state
        .map
        .tiles
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            let r = t.resource.filter(|r| is_strategic(*r))?;
            (t.owner == Some(p) && known_to(r, &player.techs)).then(|| {
                let at = state.map.hex_at(i);
                let centre = t
                    .city
                    .and_then(|c| state.cities.get(&c))
                    .is_some_and(|c| c.pos == at);
                Deposit {
                    resource: r,
                    at,
                    city: t.city,
                    mined: t.improved || centre,
                }
            })
        })
        .collect()
}

/// Total supply of `resource` this empire's deposits give.
pub fn supplied(state: &GameState, p: PlayerId, resource: Resource) -> i32 {
    if !is_strategic(resource) || !known_to(resource, &state.player(p).techs) {
        return 0;
    }
    state.map.tiles.iter().enumerate().fold(0, |total, (i, t)| {
        if t.resource != Some(resource) || t.owner != Some(p) {
            return total;
        }
        let centre = t
            .city
            .and_then(|c| state.cities.get(&c))
            .is_some_and(|c| c.pos == state.map.hex_at(i));
        total
            + deposit_supply(
                state.player(p).faction,
                state.setup.rule_experiments.faction_rules_off,
                t.improved || centre,
            )
    })
}

/// The building tier's own need, its era's resource (`SUPPLY_TIERS`): Forge and
/// University Iron, Bank Niter, Factory Coal, Research Lab Oil (the Stock Exchange none). A higher tier keeps its lower
/// tiers' needs (a Factory city holds the Forge tier below it, so it uses Iron and Coal). Walls,
/// food buildings, Roadworks, Great Works and faction landmarks need nothing.
pub fn building_need(building: BuildingKind) -> Option<Resource> {
    if SUPPLY_TIERS.contains(&building) {
        era_resource(
            data::building_stats(building)
                .requires
                .map_or(crate::eras::FIRST_ERA, crate::eras::era_of_tech),
        )
    } else {
        None
    }
}

/// Fusion releases the Factory and Research Lab tiers from their own supply. Lower tiers
/// continue to use their historical resources.
pub fn building_need_for(player: &Player, building: BuildingKind) -> Option<Resource> {
    if player.techs.contains(&TechId::FusionPower)
        && matches!(
            building,
            BuildingKind::Autoworks | BuildingKind::InferenceCore
        )
    {
        None
    } else {
        building_need(building)
    }
}

pub fn item_need_for(player: &Player, item: ProductionItem) -> Option<Resource> {
    match item {
        ProductionItem::Building { building } => building_need_for(player, building),
        _ => item_need(item),
    }
}

/// The supply each Siege unit uses (`data::SIEGE_USES_IRON`), or none.
pub fn siege_need() -> Option<Resource> {
    data::SIEGE_USES_IRON.then_some(Resource::Iron)
}

/// A military unit, including an aircraft in a city hangar, uses its era's resource from the
/// Classical era on. Ancient units (all on offer from turn 1) and civilians need nothing.
pub fn unit_need(unit: UnitKind) -> Option<Resource> {
    data::unit_class(unit)?;
    let era = data::unit_era_tier(unit);
    era_resource(era)
}

/// The supply an item adds when it exists (one of the resource, for as long as it stands).
pub fn item_need(item: ProductionItem) -> Option<Resource> {
    match item {
        ProductionItem::Unit { unit } => unit_need(unit),
        ProductionItem::Building { building } => building_need(building),
        _ => None,
    }
}

/// The supply an upgrade from `from` adds (the target's need, when `from` lacks it).
pub fn upgrade_need(from: UnitKind) -> Option<Resource> {
    let to = data::unit_stats(from).upgrades_to?;
    unit_need(to).filter(|r| unit_need(from) != Some(*r))
}

/// One thing that uses a supply while it exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SupplyUser {
    /// A building tier in a city (`MachineShop` for a Forge's Iron, `Autoworks` for a
    /// Factory's Coal).
    Building {
        city: CityId,
        building: BuildingKind,
    },
    Unit {
        unit: UnitId,
    },
    /// The Siege bought for a unit, which uses its own supply on top of the unit's.
    Siege {
        unit: UnitId,
    },
}

/// An empire's record of one supply user: since when it has used the supply (newest go idle
/// first) and whether it is idle now. Kept sorted by `user`; rewritten by `update_supply`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyRecord {
    pub user: SupplyUser,
    pub resource: Resource,
    /// The turn this empire first had this user (built, upgraded or captured).
    pub since: u32,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub idle: bool,
}

/// What uses supply in this empire now, in `SupplyUser` order.
pub fn current_users(state: &GameState, p: PlayerId) -> Vec<(SupplyUser, Resource)> {
    let mut out = Vec::new();
    for c in state.cities_of(p) {
        for tier in SUPPLY_TIERS {
            if data::has_building(&c.buildings, tier)
                && let Some(r) = building_need_for(state.player(p), tier)
            {
                out.push((
                    SupplyUser::Building {
                        city: c.id,
                        building: tier,
                    },
                    r,
                ));
            }
        }
    }
    // A hired Company brings its seller's supply: it uses none of its employer's.
    for u in state.units_of(p).filter(|u| !crate::company::is_company(u)) {
        if let Some(r) = unit_need(u.kind) {
            out.push((SupplyUser::Unit { unit: u.id }, r));
        }
        if u.siege
            && let Some(r) = siege_need()
        {
            out.push((SupplyUser::Siege { unit: u.id }, r));
        }
    }
    // Aircraft live in hangars, not on the map; they share the unit id space.
    for a in state.aircraft.iter() {
        if state.cities.get(&a.city).is_some_and(|c| c.owner == p)
            && let Some(r) = unit_need(a.kind)
        {
            out.push((SupplyUser::Unit { unit: a.id }, r));
        }
    }
    out.sort();
    out
}

/// How many of this empire's buildings and units use `resource` now.
pub fn in_use(state: &GameState, p: PlayerId, resource: Resource) -> i32 {
    let buildings: i32 = state
        .cities_of(p)
        .map(|c| {
            SUPPLY_TIERS
                .into_iter()
                .filter(|&tier| {
                    data::has_building(&c.buildings, tier)
                        && building_need_for(state.player(p), tier) == Some(resource)
                })
                .count() as i32
        })
        .sum();
    // A Company uses its seller's supply. Siege is an additional user of the same unit.
    let units: i32 = state
        .units_of(p)
        .filter(|u| !crate::company::is_company(u))
        .map(|u| {
            i32::from(unit_need(u.kind) == Some(resource))
                + i32::from(u.siege && siege_need() == Some(resource))
        })
        .sum();
    let aircraft = state
        .aircraft
        .iter()
        .filter(|a| state.cities.get(&a.city).is_some_and(|c| c.owner == p))
        .filter(|a| unit_need(a.kind) == Some(resource))
        .count() as i32;
    buildings + units + aircraft
}

/// Builds in progress that will use `resource` when they finish, outside `except` (a city
/// whose own build is being replaced or bought). They hold their supply while they are built.
pub fn reserved(
    state: &GameState,
    p: PlayerId,
    resource: Resource,
    except: Option<CityId>,
) -> Vec<(CityId, ProductionItem)> {
    state
        .cities_of(p)
        .filter(|c| Some(c.id) != except)
        .filter_map(|c| {
            let item = c.producing?;
            (item_need_for(state.player(p), item) == Some(resource)).then_some((c.id, item))
        })
        .collect()
}

/// Supply left for a new user: supplied, minus what is in use and what builds hold.
pub fn spare(state: &GameState, p: PlayerId, resource: Resource, except: Option<CityId>) -> i32 {
    supplied(state, p, resource)
        - in_use(state, p, resource)
        - reserved(state, p, resource, except).len() as i32
}

/// Counts for one player's view of one fixed state. City options ask about the same resources
/// repeatedly; resolution and order validation continue to use the uncached functions.
pub(crate) struct ViewSupply {
    player: PlayerId,
    supplied: [i32; Resource::ALL.len()],
    in_use: [i32; Resource::ALL.len()],
    reserved: [i32; Resource::ALL.len()],
}

impl ViewSupply {
    pub(crate) fn new(state: &GameState, player: PlayerId) -> Self {
        let mut supplied_counts = [0; Resource::ALL.len()];
        let mut used_counts = [0; Resource::ALL.len()];
        let mut reserved_counts = [0; Resource::ALL.len()];
        for resource in Resource::ALL {
            supplied_counts[resource as usize] = supplied(state, player, resource);
            used_counts[resource as usize] = in_use(state, player, resource);
        }
        for city in state.cities_of(player) {
            if let Some(resource) = city
                .producing
                .and_then(|item| item_need_for(state.player(player), item))
            {
                reserved_counts[resource as usize] += 1;
            }
        }
        Self {
            player,
            supplied: supplied_counts,
            in_use: used_counts,
            reserved: reserved_counts,
        }
    }

    fn supplied(&self, player: PlayerId, resource: Resource) -> i32 {
        debug_assert_eq!(self.player, player);
        self.supplied[resource as usize]
    }

    fn spare(
        &self,
        state: &GameState,
        player: PlayerId,
        resource: Resource,
        except: Option<CityId>,
    ) -> i32 {
        debug_assert_eq!(self.player, player);
        let returned = except
            .and_then(|id| state.cities.get(&id))
            .filter(|city| city.owner == player)
            .and_then(|city| {
                let item = city.producing?;
                (item_need_for(state.player(player), item) == Some(resource)).then_some(1)
            })
            .unwrap_or(0);
        let cached = self.supplied[resource as usize]
            - self.in_use[resource as usize]
            - self.reserved[resource as usize]
            + returned;
        debug_assert_eq!(cached, spare(state, player, resource, except));
        cached
    }
}

fn record(player: &Player, user: SupplyUser) -> Option<&SupplyRecord> {
    player
        .supply
        .binary_search_by(|r| r.user.cmp(&user))
        .ok()
        .map(|i| &player.supply[i])
}

pub(crate) fn is_idle(player: &Player, user: SupplyUser) -> bool {
    record(player, user).is_some_and(|r| r.idle)
}

/// Why this building tier in `city` yields nothing this turn: the resource it is short of.
/// A tier works only while every supplied tier below it does too (a Factory needs its Forge).
pub fn idle_tier(player: &Player, city: CityId, tier: BuildingKind) -> Option<Resource> {
    let mut at = Some(tier);
    while let Some(building) = at {
        if let Some(r) = building_need_for(player, building)
            && is_idle(player, SupplyUser::Building { city, building })
        {
            return Some(r);
        }
        at = data::building_predecessor(building);
    }
    None
}

/// The idle building tiers of a city, lowest tier first.
pub fn idle_tiers(player: &Player, city: &City) -> Vec<(BuildingKind, Resource)> {
    if player.supply.is_empty() {
        return Vec::new();
    }
    SUPPLY_TIERS
        .into_iter()
        .filter(|t| data::has_building(&city.buildings, *t))
        .filter_map(|t| idle_tier(player, city.id, t).map(|r| (t, r)))
        .collect()
}

/// The resource this unit is short of, if its owner's supply can't cover it.
pub fn unit_unsupplied(state: &GameState, unit: &Unit) -> Option<Resource> {
    let player = state.player(unit.owner);
    unit_need(unit.kind)
        .filter(|_| is_idle(player, SupplyUser::Unit { unit: unit.id }))
        .or_else(|| {
            siege_need()
                .filter(|_| unit.siege)
                .filter(|_| is_idle(player, SupplyUser::Siege { unit: unit.id }))
        })
}

/// Strength lost by an unsupplied unit: `UNSUPPLIED_STRENGTH_PERCENT` of its base strength.
pub fn unsupplied_penalty(base: i32) -> i32 {
    base * UNSUPPLIED_STRENGTH_PERCENT / 100
}

/// The strength line of an unsupplied unit: "No Iron supply −8" (`UNSUPPLIED_STRENGTH_PERCENT`
/// of its base strength, or of its ranged strength when it shoots).
pub fn add_unsupplied(
    b: &mut Breakdown,
    faction: Faction,
    kind: UnitKind,
    shooting: bool,
    missing: Option<Resource>,
) {
    if let Some(r) = missing {
        let stats = data::unit_stats_for(faction, kind);
        let base = if shooting {
            stats.ranged
        } else {
            stats.strength
        };
        b.add(format!("No {} supply", name(r)), -unsupplied_penalty(base));
    }
}

/// Count supply at the end of a turn: every user keeps its `since`, new ones start now, and
/// for each resource the newest users beyond the supply go idle until it returns.
pub fn update_supply(state: &mut GameState, events: &mut Vec<Event>) {
    let ids: Vec<PlayerId> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    for p in ids {
        let users = current_users(state, p);
        let player = state.player(p);
        if users.is_empty() && player.supply.is_empty() {
            continue;
        }
        let turn = state.turn;
        let mut records: Vec<SupplyRecord> = users
            .iter()
            .map(|&(user, resource)| SupplyRecord {
                user,
                resource,
                // A unit that upgraded into another era's resource is a new user of it.
                since: record(player, user)
                    .filter(|r| r.resource == resource)
                    .map_or(turn, |r| r.since),
                idle: false,
            })
            .collect();
        let mut changes = Vec::new();
        for resource in Resource::STRATEGIC {
            let supply = supplied(state, p, resource);
            let mut order: Vec<usize> = (0..records.len())
                .filter(|i| records[*i].resource == resource)
                .collect();
            order.sort_by_key(|i| (records[*i].since, records[*i].user));
            for (rank, i) in order.iter().enumerate() {
                records[*i].idle = rank as i32 >= supply;
            }
            let idle = order.iter().filter(|i| records[**i].idle).count() as u32;
            let was = player
                .supply
                .iter()
                .filter(|r| r.resource == resource && r.idle)
                .count() as u32;
            if idle != was {
                changes.push(Event::SupplyShortage {
                    player: p,
                    resource,
                    idle,
                    supplied: supply,
                    used: order.len() as u32,
                });
            }
        }
        state.player_mut(p).supply = records;
        events.extend(changes);
    }
}

/// What starting `item` in `city` would need that the owner lacks (pillar 5). The city's own
/// build in progress gives its supply back when it is replaced, so it doesn't count.
pub fn start_blocker(state: &GameState, city: &City, item: ProductionItem) -> Option<String> {
    start_check(state, city, item)
}

/// `start_blocker`, worded or not (`economy::Reason`).
pub fn start_check<R: crate::economy::Reason>(
    state: &GameState,
    city: &City,
    item: ProductionItem,
) -> Option<R> {
    start_check_with_supply(state, city, item, None)
}

pub(crate) fn start_check_with_supply<R: crate::economy::Reason>(
    state: &GameState,
    city: &City,
    item: ProductionItem,
    supply: Option<&ViewSupply>,
) -> Option<R> {
    let need = item_need_for(state.player(city.owner), item)?;
    if let ProductionItem::Building { building } = item
        && let Some(below) = data::building_predecessor(building)
        && let Some(r) = idle_tier(state.player(city.owner), city.id, below)
    {
        return Some(R::word(|| {
            let faction = state.player(city.owner).faction;
            let mut lower = below;
            while building_need_for(state.player(city.owner), lower) != Some(r)
                || !is_idle(
                    state.player(city.owner),
                    SupplyUser::Building {
                        city: city.id,
                        building: lower,
                    },
                )
            {
                lower = data::building_predecessor(lower).expect("an idle tier below");
            }
            let lower = data::building_name_for(faction, lower);
            let item = with_article_name(data::building_name_for(faction, building));
            let mut item = item.chars();
            let first = item.next().map(|c| c.to_ascii_uppercase()).unwrap_or('A');
            format!(
                "This city's {lower} is idle (no {}). {first}{} needs a working {lower}",
                name(r),
                item.as_str()
            )
        }));
    }
    shortage_after_with_supply(state, city.owner, need, Some(city.id), 0, supply)
}

/// A purchase is a new user too. Buying the build in progress takes over its held supply.
pub fn buy_blocker(state: &GameState, city: &City, item: ProductionItem) -> Option<String> {
    buy_check(state, city, item)
}

/// `buy_blocker`, worded or not (`economy::Reason`).
pub fn buy_check<R: crate::economy::Reason>(
    state: &GameState,
    city: &City,
    item: ProductionItem,
) -> Option<R> {
    buy_check_with_supply(state, city, item, None)
}

pub(crate) fn buy_check_with_supply<R: crate::economy::Reason>(
    state: &GameState,
    city: &City,
    item: ProductionItem,
    supply: Option<&ViewSupply>,
) -> Option<R> {
    let need = item_need_for(state.player(city.owner), item)?;
    let except = (city.producing == Some(item)).then_some(city.id);
    shortage_after_with_supply(state, city.owner, need, except, 0, supply)
}

/// Why an upgrade can't be supplied, with the same wording as production. `earlier` counts
/// upgrades earlier in the same order list that use the same resource.
pub fn upgrade_blocker(
    state: &GameState,
    p: PlayerId,
    unit: &Unit,
    to: UnitKind,
    earlier: i32,
) -> Option<String> {
    let need = unit_upgrade_need_to(unit, to)?;
    shortage_after(state, p, need, None, earlier)
}

/// `shortage` for an order list: `earlier` supply users ahead of this one in the same list are
/// counted too.
pub fn shortage_after_orders(
    state: &GameState,
    p: PlayerId,
    resource: Resource,
    earlier: i32,
) -> Option<String> {
    shortage_after(state, p, resource, None, earlier)
}

/// The supply upgrading this unit adds.
pub fn unit_upgrade_need(unit: &Unit) -> Option<Resource> {
    unit_upgrade_need_to(unit, data::unit_stats(unit.kind).upgrades_to?)
}

/// Additional supply for the final form of a multi-step upgrade.
pub fn unit_upgrade_need_to(unit: &Unit, to: UnitKind) -> Option<Resource> {
    unit_need(to).filter(|need| unit_need(unit.kind) != Some(*need))
}

/// "No Iron to spare: …" when no supply is left for one more user.
pub fn shortage<R: crate::economy::Reason>(
    state: &GameState,
    p: PlayerId,
    resource: Resource,
    except: Option<CityId>,
) -> Option<R> {
    shortage_after(state, p, resource, except, 0)
}

fn shortage_after<R: crate::economy::Reason>(
    state: &GameState,
    p: PlayerId,
    resource: Resource,
    except: Option<CityId>,
    earlier: i32,
) -> Option<R> {
    shortage_after_with_supply(state, p, resource, except, earlier, None)
}

fn shortage_after_with_supply<R: crate::economy::Reason>(
    state: &GameState,
    p: PlayerId,
    resource: Resource,
    except: Option<CityId>,
    earlier: i32,
    supply: Option<&ViewSupply>,
) -> Option<R> {
    let player = state.player(p);
    if !known_to(resource, &player.techs) {
        return Some(R::word(|| {
            let r = name(resource);
            let tech = stats(resource).revealed_by.map_or("", data::tech_name);
            format!("Needs {r}: {tech} reveals {r} on the map")
        }));
    }
    if supply.map_or_else(
        || spare(state, p, resource, except),
        |s| s.spare(state, p, resource, except),
    ) - earlier
        >= 1
    {
        return None;
    }
    Some(R::word(|| {
        shortage_text(state, p, resource, except, earlier, supply)
    }))
}

/// The improvement a deposit of `resource` takes, by name ("Mine", "Oil Well").
pub fn improvement_name(resource: Resource) -> &'static str {
    crate::improvements::name(crate::improvements::kind_for(resource), false)
}

/// "an Iron", "a Coal".
pub fn with_article(resource: Resource) -> String {
    with_article_name(name(resource))
}

/// "an Oil", "a Factory".
pub fn with_article_name(r: &str) -> String {
    let vowel = r.starts_with(['A', 'E', 'I', 'O', 'U']);
    format!("{} {r}", if vowel { "an" } else { "a" })
}

/// The fix, said the same way everywhere: "Bring another Iron deposit inside your borders, or
/// build a Mine on one for +1 Iron." (each resource's own improvement: "an Oil Well" for Oil).
pub fn fix_clause(resource: Resource, any: bool, mine_gain: i32) -> String {
    let r = name(resource);
    let deposit = if any {
        format!("another {r} deposit")
    } else {
        format!("{} deposit", with_article(resource))
    };
    format!(
        "Bring {deposit} inside your borders, or build {} on one for +{mine_gain} {r}.",
        with_article_name(improvement_name(resource))
    )
}

fn shortage_text(
    state: &GameState,
    p: PlayerId,
    resource: Resource,
    except: Option<CityId>,
    _earlier: i32,
    view_supply: Option<&ViewSupply>,
) -> String {
    let r = name(resource);
    let supply =
        view_supply.map_or_else(|| supplied(state, p, resource), |s| s.supplied(p, resource));
    if supply == 0 {
        return format!(
            "Needs 1 {r}: you have none yet. {}",
            fix_clause(
                resource,
                false,
                mine_supply_gain(
                    state.player(p).faction,
                    state.setup.rule_experiments.faction_rules_off
                )
            )
        );
    }
    let held = reserved(state, p, resource, except);
    let mut text = format!("No {r} to spare: all {supply} of your {r} are in use.");
    if !held.is_empty() {
        let names: Vec<String> = held
            .iter()
            .map(|(c, item)| {
                format!(
                    "the {} being built in {}",
                    crate::rules::cities::item_name(*item),
                    state.cities[c].name
                )
            })
            .collect();
        text.push_str(&format!(
            " {} {} held for {}.",
            held.len(),
            if held.len() == 1 { "is" } else { "are" },
            names.join(" and ")
        ));
    }
    text.push(' ');
    text.push_str(&fix_clause(
        resource,
        true,
        mine_supply_gain(
            state.player(p).faction,
            state.setup.rule_experiments.faction_rules_off,
        ),
    ));
    text
}

/// Supply per (player, resource, city), used to announce newly supplying deposits.
pub type SourceCounts = BTreeMap<(PlayerId, Resource, Option<CityId>), (u32, i32)>;

pub fn source_counts(state: &GameState) -> SourceCounts {
    let mut counts = SourceCounts::new();
    for p in state.players.iter().filter(|p| p.alive) {
        for d in deposits(state, p.id) {
            let e = counts.entry((p.id, d.resource, d.city)).or_default();
            e.0 += 1;
            e.1 += deposit_supply(
                p.faction,
                state.setup.rule_experiments.faction_rules_off,
                d.mined,
            );
        }
    }
    counts
}

/// Team-private report lines: "Iron found near Brasslock: +2 Iron supply", and a reveal
/// with no deposit inside your borders yet.
pub fn announce(before: &SourceCounts, state: &GameState, events: &mut Vec<Event>) {
    let after = source_counts(state);
    for (&(player, resource, city), &(n, supply)) in &after {
        let (old, old_supply) = before
            .get(&(player, resource, city))
            .copied()
            .unwrap_or((0, 0));
        if n > old {
            events.push(Event::ResourcesRevealed {
                player,
                resource,
                city,
                sources: n - old,
                supply: supply - old_supply,
            });
        }
    }
    let learned: Vec<(PlayerId, TechId)> = events
        .iter()
        .filter_map(|e| match e {
            Event::TechResearched { player, tech } => Some((*player, *tech)),
            _ => None,
        })
        .collect();
    for (player, tech) in learned {
        for resource in Resource::STRATEGIC {
            if stats(resource).revealed_by == Some(tech)
                && !after.keys().any(|(p, r, _)| *p == player && *r == resource)
            {
                events.push(Event::ResourcesRevealed {
                    player,
                    resource,
                    city: None,
                    sources: 0,
                    supply: 0,
                });
            }
        }
    }
}

/// One strategic supply as its owner sees it (the top bar chip "Iron 5 / 6" and its tooltip).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyView {
    pub resource: Resource,
    pub revealed_by: Option<TechId>,
    /// The viewer knows the revealing technology.
    pub revealed: bool,
    /// What the deposits give, one line per deposit ("Iron near Lugano, with a Mine": 3).
    pub supplied: Breakdown,
    /// The same deposits as data, in map order, so the UI can name each by its tile.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deposits: Vec<DepositView>,
    /// What uses it, one line per building or group of units ("Forge in Lugano": 1).
    pub used: Breakdown,
    /// Users short of supply, newest first: they yield nothing (buildings) or fight weaker
    /// (units) until supply returns.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub idle: Vec<String>,
    /// Builds in progress holding supply for when they finish ("Forge in Bern").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub held: Vec<String>,
    /// Supply left for one more user: supplied − used − held.
    pub spare: i32,
}

fn user_label(state: &GameState, user: SupplyUser) -> String {
    match user {
        SupplyUser::Building { city, building } => {
            let faction = state
                .cities
                .get(&city)
                .map_or(Faction::Byzantium, |c| state.player(c.owner).faction);
            format!(
                "{} in {}",
                data::building_name_for(faction, building),
                city_name(state, Some(city))
            )
        }
        SupplyUser::Unit { unit } => state.units.get(&unit).map_or_else(
            || {
                state.aircraft.iter().find(|a| a.id == unit).map_or_else(
                    || "Unit".into(),
                    |a| {
                        let owner = state.cities[&a.city].owner;
                        data::unit_name_for(state.player(owner).faction, a.kind).to_string()
                    },
                )
            },
            |u| data::unit_name_for(state.made_by(u), u.kind).to_string(),
        ),
        SupplyUser::Siege { unit } => state.units.get(&unit).map_or_else(
            || "Siege".into(),
            |u| {
                format!(
                    "{} with Siege",
                    data::unit_name_for(state.made_by(u), u.kind)
                )
            },
        ),
    }
}

fn city_name(state: &GameState, city: Option<CityId>) -> String {
    city.and_then(|c| state.cities.get(&c))
        .map_or_else(|| "your land".into(), |c| c.name.clone())
}

/// One supplying deposit as its owner sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositView {
    pub hex: Hex,
    /// The city whose land it is on (None: land no city works).
    pub city: Option<CityId>,
    pub mined: bool,
    pub supply: i32,
}

pub fn supply_views(state: &GameState, p: PlayerId) -> Vec<SupplyView> {
    let player = state.player(p);
    let deposits = deposits(state, p);
    let users = current_users(state, p);
    Resource::STRATEGIC
        .into_iter()
        .map(|resource| {
            let r = name(resource);
            let mut supplied = Breakdown::default();
            for d in deposits.iter().filter(|d| d.resource == resource) {
                supplied.add(
                    format!(
                        "{r} near {}{}",
                        city_name(state, d.city),
                        if d.mined {
                            format!(", with {}", with_article_name(improvement_name(resource)))
                        } else {
                            String::new()
                        }
                    ),
                    deposit_supply(
                        player.faction,
                        state.setup.rule_experiments.faction_rules_off,
                        d.mined,
                    ),
                );
            }
            let mut used = Breakdown::default();
            let mut units: BTreeMap<String, i32> = BTreeMap::new();
            for (user, _) in users.iter().filter(|(_, x)| *x == resource) {
                match user {
                    SupplyUser::Building { .. } => used.add(user_label(state, *user), 1),
                    SupplyUser::Unit { .. } | SupplyUser::Siege { .. } => {
                        *units.entry(user_label(state, *user)).or_default() += 1
                    }
                }
            }
            for (label, n) in units {
                used.add(
                    if n == 1 {
                        label
                    } else {
                        format!("{label} ×{n}")
                    },
                    n,
                );
            }
            let mut idle: Vec<&SupplyRecord> = player
                .supply
                .iter()
                .filter(|x| x.resource == resource && x.idle)
                .collect();
            idle.sort_by_key(|x| std::cmp::Reverse((x.since, x.user)));
            let held: Vec<String> = reserved(state, p, resource, None)
                .into_iter()
                .map(|(c, item)| {
                    format!(
                        "{} in {}",
                        crate::rules::cities::item_name(item),
                        state.cities[&c].name
                    )
                })
                .collect();
            SupplyView {
                deposits: deposits
                    .iter()
                    .filter(|d| d.resource == resource)
                    .map(|d| DepositView {
                        hex: d.at,
                        city: d.city,
                        mined: d.mined,
                        supply: deposit_supply(
                            player.faction,
                            state.setup.rule_experiments.faction_rules_off,
                            d.mined,
                        ),
                    })
                    .collect(),
                resource,
                revealed_by: stats(resource).revealed_by,
                revealed: known_to(resource, &player.techs),
                spare: supplied.total
                    - used.total
                    - reserved(state, p, resource, None).len() as i32,
                supplied,
                used,
                idle: idle.iter().map(|x| user_label(state, x.user)).collect(),
                held,
            }
        })
        .collect()
}

/// The resource a viewer sees on a tile: bonus resources wherever the tile is explored,
/// strategic ones only once the viewer knows the revealing technology. Deposits never move,
/// so an explored tile remembers its resource exactly as it remembers its terrain.
pub fn visible_resource(tile: &Tile, explored: bool, techs: &BTreeSet<TechId>) -> Option<Resource> {
    tile.resource.filter(|r| explored && known_to(*r, techs))
}

#[cfg(test)]
mod tests;

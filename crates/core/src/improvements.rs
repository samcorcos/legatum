//! Tile improvements (playtest 8, user decision 2026-10-04): no Builder units. A city builds an
//! improvement as a production item that names one tile it owns and works. A resource tile
//! yields its bonus only once improved, and tile improvements upgrade by themselves with
//! technology (no re-building).
//!
//! The rules in one place:
//! - Four improvements cover the six resources: Farm (Wheat), Pasture (Cattle), Quarry (Stone),
//!   Mine (Copper, Iron, Coal). Only resource tiles can be improved; the resource must be known
//!   to the owner, so a hidden strategic deposit can't be targeted.
//! - The target tile belongs to the building city and lies within `CITY_WORK_RADIUS` of it (only
//!   worked tiles can be improved; tiles claimed beyond that block rivals and yield nothing).
//! - An improved resource tile yields the resource's bonus, plus one more of its main yield once
//!   the owner knows the improvement's upgrade technology (`upgrade_tech`).
//! - Supply: a known strategic deposit inside the borders supplies 2, and a Mine on it adds 1
//!   (`resources::SUPPLY_PER_MINE`).
//! - Releasing the tile, razing its city or founding a city on it loses its improvement
//!   (`Tile::improved`); a captured city's improved land passes to the captor.

use crate::data;
use crate::economy;
use crate::hex::Hex;
use crate::resources::{self, Resource};
use crate::state::{City, GameState, Tile};
use crate::types::*;

impl ImprovementKind {
    pub const ALL: [ImprovementKind; 7] = [
        ImprovementKind::Farm,
        ImprovementKind::Pasture,
        ImprovementKind::Quarry,
        ImprovementKind::Mine,
        ImprovementKind::SaltpetreWorks,
        ImprovementKind::OilWell,
        ImprovementKind::QuartzMine,
    ];
}

/// The improvement a resource needs.
pub fn kind_for(resource: Resource) -> ImprovementKind {
    match resource {
        Resource::Wheat => ImprovementKind::Farm,
        Resource::Cattle => ImprovementKind::Pasture,
        Resource::Stone => ImprovementKind::Quarry,
        Resource::Copper | Resource::Iron | Resource::Coal => ImprovementKind::Mine,
        Resource::Niter => ImprovementKind::SaltpetreWorks,
        Resource::Oil => ImprovementKind::OilWell,
        Resource::Silicon => ImprovementKind::QuartzMine,
    }
}

/// The improvement's name before and after its upgrade.
pub fn name(kind: ImprovementKind, upgraded: bool) -> &'static str {
    match (kind, upgraded) {
        (ImprovementKind::Farm, false) => "Farm",
        (ImprovementKind::Farm, true) => "Irrigated Farm",
        (ImprovementKind::Pasture, false) => "Pasture",
        (ImprovementKind::Pasture, true) => "Ranch",
        (ImprovementKind::Quarry, false) => "Quarry",
        (ImprovementKind::Quarry, true) => "Deep Quarry",
        (ImprovementKind::Mine, false) => "Mine",
        (ImprovementKind::Mine, true) => "Deep Mine",
        // The later eras' improvements come after Iron Working, so their upgrade is built in.
        (ImprovementKind::SaltpetreWorks, _) => "Saltpetre Works",
        (ImprovementKind::OilWell, _) => "Oil Well",
        (ImprovementKind::QuartzMine, _) => "Quartz Mine",
    }
}

/// Production cost, before faction premiums.
pub fn base_cost(kind: ImprovementKind) -> i32 {
    match kind {
        ImprovementKind::Farm | ImprovementKind::Pasture | ImprovementKind::Quarry => 30,
        ImprovementKind::Mine
        | ImprovementKind::SaltpetreWorks
        | ImprovementKind::OilWell
        | ImprovementKind::QuartzMine => 40,
    }
}

/// The technology that upgrades this improvement in place.
pub fn upgrade_tech(kind: ImprovementKind) -> TechId {
    match kind {
        ImprovementKind::Farm | ImprovementKind::Pasture => TechId::CropRotation,
        ImprovementKind::Quarry => TechId::Masonry,
        ImprovementKind::OilWell => TechId::Combustion,
        ImprovementKind::Mine | ImprovementKind::SaltpetreWorks | ImprovementKind::QuartzMine => {
            TechId::IronWorking
        }
    }
}

/// Whether an owner with this knowledge has the upgraded version.
pub fn upgraded(kind: ImprovementKind, knows: impl Fn(TechId) -> bool) -> bool {
    knows(upgrade_tech(kind))
}

/// The upgrade's extra yield: one more of the resource's main yield.
pub fn upgrade_yields(resource: Resource) -> Yields {
    if resource == Resource::Oil {
        return Yields::new(0, 0, 1, 0);
    }
    let b = resources::stats(resource).bonus;
    if b.food > 0 {
        Yields::new(1, 0, 0, 0)
    } else if b.production > 0 {
        Yields::new(0, 1, 0, 0)
    } else {
        Yields::new(0, 0, 1, 0)
    }
}

/// What an improved resource adds to its tile for an owner with this knowledge.
pub fn resource_yields(resource: Resource, knows: impl Fn(TechId) -> bool) -> Yields {
    if !resources::known(resource, &knows) {
        return Yields::ZERO;
    }
    let mut y = resources::stats(resource).bonus;
    if upgraded(kind_for(resource), &knows) {
        y += upgrade_yields(resource);
    }
    if matches!(
        resource,
        Resource::Copper | Resource::Iron | Resource::Coal | Resource::Silicon
    ) && knows(TechId::Railways)
    {
        y.production += 1;
    }
    y
}

/// The resource part of a tile's yield: nothing until the tile is improved. A city centre counts
/// as improved (the city is its improvement).
pub fn tile_resource_yields(tile: &Tile, centre: bool, knows: impl Fn(TechId) -> bool) -> Yields {
    match tile.resource {
        Some(r) if tile.improved || centre => resource_yields(r, knows),
        _ => Yields::ZERO,
    }
}

/// Why `city` can't build `kind` on `tile`, or `None` when it can.
pub fn blocker(state: &GameState, city: &City, kind: ImprovementKind, tile: Hex) -> Option<String> {
    check(state, city, kind, tile)
}

/// `blocker`, worded or not (`economy::Reason`).
pub fn check<R: economy::Reason>(
    state: &GameState,
    city: &City,
    kind: ImprovementKind,
    tile: Hex,
) -> Option<R> {
    let Some(t) = state.map.get(tile) else {
        return Some(R::word(|| "That tile is off the map".into()));
    };
    if t.city != Some(city.id) || t.owner != Some(city.owner) {
        return Some(R::word(|| {
            format!("That tile isn't owned by {}", city.name)
        }));
    }
    if tile == city.pos || tile.distance(city.pos) > data::CITY_WORK_RADIUS {
        return Some(R::word(|| {
            format!(
                "Only worked tiles can be improved: within {} tiles of {}",
                data::CITY_WORK_RADIUS,
                city.name
            )
        }));
    }
    if t.landmark.is_some() {
        return Some(R::word(|| "A wonder occupies that tile".into()));
    }
    let player = state.player(city.owner);
    let Some(resource) = t
        .resource
        .filter(|r| resources::known_to(*r, &player.techs))
    else {
        return Some(R::word(|| "There is no known resource on that tile".into()));
    };
    if kind_for(resource) != kind {
        return Some(R::word(|| {
            format!(
                "{} needs a {}, not a {}",
                resources::name(resource),
                name(kind_for(resource), false),
                name(kind, false)
            )
        }));
    }
    if t.improved {
        return Some(R::word(|| {
            format!("{} is already improved", resources::name(resource))
        }));
    }
    None
}

/// Every improvement this city could build now, in hex order: its worked resource tiles that are
/// known, unimproved and unoccupied.
pub fn candidates(state: &GameState, city: &City) -> Vec<ProductionItem> {
    let mut out = Vec::new();
    for h in city.pos.within(data::CITY_WORK_RADIUS) {
        let Some(resource) = state.map.get(h).and_then(|t| t.resource) else {
            continue;
        };
        let kind = kind_for(resource);
        if check::<economy::Blocked>(state, city, kind, h).is_none() {
            out.push(ProductionItem::Improvement {
                improvement: kind,
                tile: h,
            });
        }
    }
    out
}

/// Mark the tile improved (the completed item).
pub fn complete(state: &mut GameState, tile: Hex) {
    if let Some(t) = state.map.get_mut(tile) {
        t.improved = true;
    }
}

#[cfg(test)]
mod tests;

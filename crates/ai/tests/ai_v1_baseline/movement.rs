//! A last pass over this turn's move orders: predict each mover's first step the way the core
//! will (same pathfinder, same tiles avoided) and keep two of our same-class units from
//! stepping into one tile, which would bounce both of them, likely again every turn.

use crate::ai_v1_baseline::scripted::Planner;
use legatum_core::path::{find_path, land_path};
use legatum_core::state::Map;
use legatum_core::{Hex, Order, Terrain, UnitId};
use std::collections::{BTreeMap, BTreeSet};

impl Planner<'_> {
    pub fn dedupe_first_steps(&mut self) {
        let view = self.view();
        let movers: BTreeMap<UnitId, Hex> = self
            .orders
            .iter()
            .filter_map(|o| match o {
                Order::MoveUnit { unit, to } => Some((*unit, *to)),
                _ => None,
            })
            .collect();
        if movers.len() < 2 {
            return;
        }
        // The view's map; unknown tiles as open ground (cost 1).
        let mut map = Map::new(view.width, view.height, Terrain::Grassland);
        for (i, t) in view.tiles.iter().enumerate() {
            if let Some(terrain) = t.terrain {
                map.tiles[i].terrain = terrain;
            }
        }
        let n = view.tiles.len();
        let mut parked_military = vec![false; n];
        let mut parked_civilian = vec![false; n];
        let mut hostile_military = vec![false; n];
        let mut hostile_any = vec![false; n];
        for u in &view.units {
            let Some(i) = view.index(u.pos) else { continue };
            let friendly = view
                .players
                .iter()
                .find(|p| p.id == u.owner)
                .is_some_and(|p| p.team == view.team);
            if friendly {
                if !movers.contains_key(&u.id) {
                    if u.military {
                        parked_military[i] = true;
                    } else {
                        parked_civilian[i] = true;
                    }
                }
            } else {
                hostile_any[i] = true;
                hostile_military[i] |= u.military;
            }
        }
        let mut hostile_city = vec![false; n];
        for c in &self.ctx.enemy_cities {
            if let Some(i) = view.index(c.pos) {
                hostile_city[i] = true;
            }
        }

        let mut claimed: BTreeSet<(bool, Hex)> = BTreeSet::new();
        let mut demote: BTreeSet<UnitId> = BTreeSet::new();
        for (id, goto) in &movers {
            let Some(u) = self.ctx.own_units.iter().find(|u| u.id == *id) else {
                continue;
            };
            let military = u.military;
            let avoid = |h: Hex| -> bool {
                if h == *goto {
                    return false;
                }
                let Some(i) = view.index(h) else { return true };
                let parked = if military {
                    parked_military[i]
                } else {
                    parked_civilian[i]
                };
                hostile_city[i] || hostile_military[i] || (!military && hostile_any[i]) || parked
            };
            let path = find_path(&map, u.pos, *goto, |h| {
                map.move_cost(h).filter(|_| !avoid(h))
            })
            .or_else(|| land_path(&map, u.pos, *goto));
            let Some(first) = path.and_then(|p| p.first().copied()) else {
                continue;
            };
            let attack = view
                .index(first)
                .is_some_and(|i| hostile_any[i] || hostile_city[i]);
            if attack {
                continue;
            }
            if !claimed.insert((military, first)) {
                demote.insert(*id);
            }
        }
        for o in &mut self.orders {
            if let Order::MoveUnit { unit, .. } = *o
                && demote.contains(&unit)
            {
                *o = Order::SkipUnit { unit };
            }
        }
    }
}

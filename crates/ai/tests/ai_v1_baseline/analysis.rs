//! Facts derived once per turn from a PlayerView: who is hostile, where threats are, which tiles
//! are frontier, which sites are worth settling, and cheapest-path distance fields.
//!
//! Everything here iterates in a defined order (view vectors sorted by id, tiles by index).

use legatum_core::combat;
use legatum_core::data::{self, MIN_CITY_GAP};
use legatum_core::economy::tile_score;
use legatum_core::view::{CityView, PlayerView, UnitView};
use legatum_core::{CityFocus, Hex, PlayerId, UnitKind};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Enemy military within this distance of an own city threatens it.
pub const THREAT_RADIUS: i32 = 3;
/// Sites are scored over this radius (the city work radius).
pub const SITE_RADIUS: i32 = data::CITY_WORK_RADIUS;

pub const UNREACHABLE: i32 = i32::MAX;

pub struct Ctx<'a> {
    pub view: &'a PlayerView,
    pub me: PlayerId,
    /// Own units, sorted by id.
    pub own_units: Vec<&'a UnitView>,
    /// Own cities (with detail), sorted by id.
    pub own_cities: Vec<&'a CityView>,
    /// Visible units of other teams, sorted by id.
    pub enemy_units: Vec<&'a UnitView>,
    /// Known cities of other teams, sorted by id.
    pub enemy_cities: Vec<&'a CityView>,
    /// Every known city position (own, team, enemy).
    pub city_positions: Vec<Hex>,
    /// Enemy military power within THREAT_RADIUS of each own city (same order as own_cities).
    pub threat: Vec<i32>,
    /// Tiles currently holding a friendly (own or teammate) military unit.
    pub friendly_military_at: Vec<bool>,
    /// Tiles with an enemy unit or city (index-aligned with the map).
    pub hostile_at: Vec<bool>,
    /// Tiles within 2 of a visible enemy military unit.
    pub danger: Vec<bool>,
    /// Explored passable tiles next to unexplored ones, plus known ruins. Sorted by hex index.
    pub frontier: Vec<Hex>,
    /// Settling score per tile index, None where a city may not (or should not) be founded.
    pub site_score: Vec<Option<i32>>,
}

impl<'a> Ctx<'a> {
    pub fn new(view: &'a PlayerView) -> Self {
        let me = view.me;
        let mut hostile = vec![true; 256];
        for p in &view.players {
            hostile[p.id.0 as usize] = p.team != view.team;
        }
        hostile[me.0 as usize] = false;

        let mut own_units: Vec<&UnitView> = view.units.iter().filter(|u| u.owner == me).collect();
        own_units.sort_by_key(|u| u.id);
        let mut own_cities: Vec<&CityView> = view
            .cities
            .iter()
            .filter(|c| c.owner == me && c.detail.is_some())
            .collect();
        own_cities.sort_by_key(|c| c.id);
        let mut enemy_units: Vec<&UnitView> = view
            .units
            .iter()
            .filter(|u| hostile[u.owner.0 as usize])
            .collect();
        enemy_units.sort_by_key(|u| u.id);
        let mut enemy_cities: Vec<&CityView> = view
            .cities
            .iter()
            .filter(|c| hostile[c.owner.0 as usize])
            .collect();
        enemy_cities.sort_by_key(|c| c.id);
        let mut city_positions: Vec<Hex> = view.cities.iter().map(|c| c.pos).collect();
        city_positions.sort();

        let n = view.tiles.len();
        let mut friendly_military_at = vec![false; n];
        let mut hostile_at = vec![false; n];
        let mut danger = vec![false; n];
        for u in &view.units {
            let Some(i) = view.index(u.pos) else { continue };
            if hostile[u.owner.0 as usize] {
                hostile_at[i] = true;
                if u.military && u.kind != UnitKind::Scout {
                    for h in u.pos.within(2) {
                        if let Some(j) = view.index(h) {
                            danger[j] = true;
                        }
                    }
                }
            } else if u.military {
                friendly_military_at[i] = true;
            }
        }
        for c in &enemy_cities {
            if let Some(i) = view.index(c.pos) {
                hostile_at[i] = true;
            }
        }

        let threat = own_cities
            .iter()
            .map(|c| {
                enemy_units
                    .iter()
                    // Scouts are too weak to count; the standing garrison handles them.
                    .filter(|u| {
                        u.military
                            && u.kind != UnitKind::Scout
                            && u.pos.distance(c.pos) <= THREAT_RADIUS
                    })
                    .map(|u| threat_power(u))
                    .sum()
            })
            .collect();

        let mut ctx = Ctx {
            view,
            me,
            own_units,
            own_cities,
            enemy_units,
            enemy_cities,
            city_positions,
            threat,
            friendly_military_at,
            hostile_at,
            danger,
            frontier: Vec::new(),
            site_score: Vec::new(),
        };
        ctx.frontier = ctx.compute_frontier();
        ctx.site_score = ctx.compute_sites();
        ctx
    }

    pub fn hex_at(&self, i: usize) -> Hex {
        let w = self.view.width as usize;
        Hex::from_offset((i % w) as i32, (i / w) as i32)
    }

    /// Explored and passable for land units: the only legal MoveUnit targets.
    pub fn passable_known(&self, h: Hex) -> bool {
        self.view
            .tile(h)
            .is_some_and(|t| t.explored && t.terrain.and_then(|t| t.move_cost()).is_some())
    }

    /// Movement cost to enter `h`: terrain cost when known, 1 (optimistic) when unknown.
    pub fn move_cost(&self, h: Hex) -> Option<i32> {
        let t = self.view.tile(h)?;
        match t.terrain {
            Some(terrain) => terrain.move_cost(),
            None => Some(1),
        }
    }

    pub fn in_danger(&self, h: Hex) -> bool {
        self.view.index(h).is_some_and(|i| self.danger[i])
    }

    /// Distance from `h` to the nearest own city, or None with no cities.
    pub fn nearest_own_city(&self, h: Hex) -> Option<&'a CityView> {
        self.own_cities
            .iter()
            .copied()
            .min_by_key(|c| (c.pos.distance(h), c.id))
    }

    pub fn capital(&self) -> Option<&'a CityView> {
        self.own_cities
            .iter()
            .copied()
            .find(|c| c.is_capital)
            .or_else(|| self.own_cities.first().copied())
    }

    fn compute_frontier(&self) -> Vec<Hex> {
        let v = self.view;
        let mut out = Vec::new();
        for (i, t) in v.tiles.iter().enumerate() {
            if !t.explored || t.terrain.and_then(|t| t.move_cost()).is_none() {
                continue;
            }
            let h = self.hex_at(i);
            let edge = h
                .neighbors()
                .iter()
                .any(|n| v.tile(*n).is_some_and(|nt| !nt.explored));
            if edge || t.ruin {
                out.push(h);
            }
        }
        out
    }

    /// How many unexplored tiles lie within 2 of `h` (what a scout would reveal there).
    pub fn unexplored_near(&self, h: Hex) -> i32 {
        h.within(2)
            .into_iter()
            .filter(|n| self.view.tile(*n).is_some_and(|t| !t.explored))
            .count() as i32
    }

    fn compute_sites(&self) -> Vec<Option<i32>> {
        let v = self.view;
        let mut out = vec![None; v.tiles.len()];
        for (i, t) in v.tiles.iter().enumerate() {
            let Some(terrain) = t.terrain else { continue };
            if !t.explored || !terrain.is_land() || terrain.move_cost().is_none() {
                continue;
            }
            if t.owner.is_some_and(|o| o != self.me) {
                continue;
            }
            let h = self.hex_at(i);
            if self
                .city_positions
                .iter()
                .any(|c| c.distance(h) <= MIN_CITY_GAP)
            {
                continue;
            }
            out[i] = self.site_score_at(h);
        }
        out
    }

    /// Sum of tile_score over known land tiles within the work radius, or None if any tile
    /// there belongs to another empire. Foreign territory within 2 usually means a city within
    /// 3, possibly one the fog hides (founded after we last saw the spot); founding there would
    /// fail at resolution, turn after turn, without the view ever showing why.
    pub fn site_score_at(&self, h: Hex) -> Option<i32> {
        let mut sum = 0;
        for n in h.within(SITE_RADIUS) {
            let Some(t) = self.view.tile(n) else { continue };
            if t.owner.is_some_and(|o| o != self.me) {
                return None;
            }
            if let Some(terrain) = t.terrain
                && t.explored
                && terrain.is_land()
            {
                sum += tile_score(terrain.yields(), CityFocus::Balanced);
            }
        }
        Some(sum)
    }

    /// Cheapest movement cost from `from` to every tile (Dijkstra; unknown tiles cost 1).
    /// `avoid_hostile` treats enemy-held tiles as impassable and adds a penalty near danger.
    pub fn distance_field(&self, from: Hex, avoid_hostile: bool) -> Vec<i32> {
        let n = self.view.tiles.len();
        let mut dist = vec![UNREACHABLE; n];
        let Some(start) = self.view.index(from) else {
            return dist;
        };
        dist[start] = 0;
        let mut open = BinaryHeap::new();
        open.push(Reverse((0i32, start)));
        while let Some(Reverse((d, i))) = open.pop() {
            if d > dist[i] {
                continue;
            }
            let h = self.hex_at(i);
            for nb in h.neighbors() {
                let Some(j) = self.view.index(nb) else {
                    continue;
                };
                let Some(mut step) = self.move_cost(nb) else {
                    continue;
                };
                if avoid_hostile {
                    if self.hostile_at[j] {
                        continue;
                    }
                    if self.danger[j] {
                        step += 4;
                    }
                }
                let nd = d + step;
                if nd < dist[j] {
                    dist[j] = nd;
                    open.push(Reverse((nd, j)));
                }
            }
        }
        dist
    }
}

/// Strength scaled by health: a unit's weight in local strength comparisons.
fn scaled(strength: i32, hp: i32, max_hp: i32) -> i32 {
    strength.max(0) * hp.max(0) / max_hp.max(1)
}

/// How dangerous an enemy unit is to things near it (its best attack, scaled by HP).
pub fn threat_power(u: &UnitView) -> i32 {
    let melee = combat::unit_attack_strength(u.kind, u.hp).total;
    let ranged = if u.ranged > 0 {
        combat::unit_ranged_strength(u.kind, u.hp).total
    } else {
        0
    };
    scaled(melee.max(ranged), u.hp, data::UNIT_HP)
}

/// A unit's melee attack weight.
pub fn attack_power(u: &UnitView) -> i32 {
    scaled(
        combat::unit_attack_strength(u.kind, u.hp).total,
        u.hp,
        data::UNIT_HP,
    )
}

/// A unit's weight when attacking: its ranged strength for ranged units, else melee.
pub fn offense_power(u: &UnitView) -> i32 {
    if u.ranged > 0 {
        scaled(
            combat::unit_ranged_strength(u.kind, u.hp).total,
            u.hp,
            data::UNIT_HP,
        )
    } else {
        attack_power(u)
    }
}

/// A unit's weight when defending its tile (terrain and fortification included).
pub fn defense_power(view: &PlayerView, u: &UnitView) -> i32 {
    let terrain = view
        .tile(u.pos)
        .and_then(|t| t.terrain)
        .unwrap_or(legatum_core::Terrain::Grassland);
    scaled(
        combat::unit_defense_strength(u.kind, u.hp, terrain, u.fortify).total,
        u.hp,
        data::UNIT_HP,
    )
}

/// A city's own defensive weight (walls included), scaled by its HP on the unit HP scale: a
/// full-health city (200 HP) weighs twice its strength, since it takes twice the damage to break.
pub fn city_power(c: &CityView) -> i32 {
    scaled(
        combat::city_defense_strength(c.population, c.walls, c.hp).total,
        c.hp,
        data::UNIT_HP,
    )
}

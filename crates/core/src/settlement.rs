//! Settlement advice from PlayerView alone. Shared by the AI and the map, including every
//! integer scoring step. The view carries only an anonymous real-city spacing mask for explored
//! tiles; no state, validation oracle, hidden terrain or unseen unit is read here.
use crate::data::{
    self, CITY_WORK_RADIUS, INDIA_CAPITAL_RADIUS, INDIA_NETWORK_PERCENT_PER_CITY,
    INDIA_NETWORK_RADIUS, india_distance_loss_percent,
};
use crate::economy::tile_score;
use crate::experiments::FactionRule;
use crate::resources;
use crate::resources::Resource;
use crate::view::{PlayerView, UnitView};
use crate::wonders::Wonder;
use crate::{CityFocus, Faction, Hex, PlayerId, Terrain, UnitKind, Yields};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub const RECOMMEND_TRAVEL_TURNS: i32 = 6;
pub const MIN_SITE_SCORE: i32 = 60;
pub const MAX_SITE_DISTANCE: i32 = 10;

/// The same predicate used by founding resolution; callers supply only the cities they know.
pub fn legal_site(
    owner: PlayerId,
    gap: i32,
    terrain: Terrain,
    tile_owner: Option<PlayerId>,
    pos: Hex,
    cities: impl IntoIterator<Item = Hex>,
) -> bool {
    terrain.is_land()
        && terrain.move_cost().is_some()
        && tile_owner.is_none_or(|o| o == owner)
        && cities.into_iter().all(|h| h.distance(pos) > gap)
}

pub fn known_legal_site(view: &PlayerView, pos: Hex) -> bool {
    view.tile(pos).is_some_and(|t| {
        t.explored
            && t.wonder.is_none()
            && t.terrain.is_some_and(|terrain| {
                legal_site(
                    view.me,
                    view.rule_experiments.min_city_gap(view.faction_of(view.me)),
                    terrain,
                    t.owner,
                    pos,
                    view.cities.iter().map(|c| c.pos),
                )
            })
    }) && !view.city_gap_blocked.contains(&pos)
        && !view.failed_city_sites.contains(&pos)
}

/// Why a tile the viewer knows is not a legal city site, in the order `check_found_city`
/// reports it. `None` exactly when `known_legal_site` holds, so the Settle lens can tint every
/// legal tile and explain every other one from the same rule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteBlocker {
    Unexplored,
    Water(Terrain),
    Impassable(Terrain),
    Wonder(Wonder),
    /// Inside another team's border.
    Foreign(PlayerId),
    /// Within the viewer's founding distance of a city (or on it, distance 0).
    TooClose {
        city: String,
        owner: PlayerId,
        distance: i32,
    },
    /// A real city is within the founding gap, but its identity is not in this view.
    CityTooClose,
    /// A founding here already failed: an unseen city must stand nearby.
    FailedBefore,
}

pub fn site_blocker(view: &PlayerView, pos: Hex) -> Option<SiteBlocker> {
    let t = view.tile(pos)?;
    let Some(terrain) = t.terrain.filter(|_| t.explored) else {
        return Some(SiteBlocker::Unexplored);
    };
    if let Some(wonder) = t.wonder {
        return Some(SiteBlocker::Wonder(wonder));
    }
    if !terrain.is_land() {
        return Some(SiteBlocker::Water(terrain));
    }
    if terrain.move_cost().is_none() {
        return Some(SiteBlocker::Impassable(terrain));
    }
    if let Some(owner) = t.owner.filter(|&o| o != view.me) {
        return Some(SiteBlocker::Foreign(owner));
    }
    if let Some(near) = view
        .cities
        .iter()
        .filter(|c| {
            c.pos.distance(pos) <= view.rule_experiments.min_city_gap(view.faction_of(view.me))
        })
        .min_by_key(|c| (c.pos.distance(pos), c.id))
    {
        return Some(SiteBlocker::TooClose {
            city: near.name.clone(),
            owner: near.owner,
            distance: near.pos.distance(pos),
        });
    }
    if view.city_gap_blocked.contains(&pos) {
        return Some(SiteBlocker::CityTooClose);
    }
    view.failed_city_sites
        .contains(&pos)
        .then_some(SiteBlocker::FailedBefore)
}

/// How good a site is on a fixed scale that means the same on every map and in every game.
/// The measure is `SettlementSite::quality` (`base_site_score`): the city's Food, Production and Gold
/// over its three-hex work radius (the outer ring at half weight), known resources, nearby
/// natural wonders, less foreign land. It ignores where the Settler stands, travel, threats and
/// the viewer's other cities, so the same tile reads the same for everyone who knows the land.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SiteBand {
    Poor,
    Fair,
    Good,
    Great,
    Exceptional,
}

impl SiteBand {
    pub const ALL: [SiteBand; 5] = [
        Self::Poor,
        Self::Fair,
        Self::Good,
        Self::Great,
        Self::Exceptional,
    ];

    /// The band a quality value falls in (`SITE_BAND_FLOORS`).
    pub fn of(quality: i32) -> Self {
        Self::ALL[SITE_BAND_FLOORS
            .iter()
            .filter(|&&floor| quality >= floor)
            .count()]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Poor => "Poor",
            Self::Fair => "Fair",
            Self::Good => "Good",
            Self::Great => "Great",
            Self::Exceptional => "Exceptional",
        }
    }
}

/// Lowest quality of Fair, Good, Great and Exceptional. Calibrated once (2026-10-11) from the
/// 309,829 legal sites of 480 generated standard maps (60 seeds for each of the 8 factions, fully
/// explored, no borders): the median site scores 175, the 10th percentile 102, the 90th 241 and the
/// 99th 303, and the factions differ by 2 points at most. These floors give Poor 18%, Fair 21%,
/// Good 30%, Great 23% and Exceptional 8% of them (`docs/learnings.md`, Settle lens bands).
/// `site_band_floors_split_generated_maps` pins the shape. Change them only with a new
/// calibration; the app reads them from `rules.generated.json` (`ui_rules` test).
pub const SITE_BAND_FLOORS: [i32; 4] = [120, 160, 200, 250];

/// A visible foreign border is evidence of a city within the work radius (claims can reach
/// farther, but most land a city claims lies within the radius it works). Mark sites that
/// could violate spacing around an unseen center consistent with that evidence. Visible
/// empty land, impassable known terrain and spacing from known cities rule centers out.
/// This is conservative advice, not a change to founding legality or a hidden-state query.
pub fn border_city_risks(view: &PlayerView) -> Vec<bool> {
    let mut centers = vec![false; view.tiles.len()];
    for (i, tile) in view.tiles.iter().enumerate() {
        if !tile.visible || !tile.owner.is_some_and(|owner| owner != view.me) {
            continue;
        }
        for center in hex_at(view, i).within(CITY_WORK_RADIUS) {
            let Some(j) = view.index(center) else {
                continue;
            };
            let possible = &view.tiles[j];
            if !possible.visible
                && possible
                    .terrain
                    .is_none_or(|t| t.is_land() && t.move_cost().is_some())
                && view.cities.iter().all(|c| {
                    c.pos.distance(center)
                        > view.rule_experiments.min_city_gap(view.faction_of(view.me))
                })
            {
                centers[j] = true;
            }
        }
    }
    let mut risks = vec![false; view.tiles.len()];
    for (i, possible) in centers.into_iter().enumerate() {
        if possible {
            for site in
                hex_at(view, i).within(view.rule_experiments.min_city_gap(view.faction_of(view.me)))
            {
                if let Some(j) = view.index(site) {
                    risks[j] = true;
                }
            }
        }
    }
    risks
}

fn hex_at(view: &PlayerView, i: usize) -> Hex {
    Hex::from_offset(i as i32 % view.width, i as i32 / view.width)
}

/// Elapsed movement points, including unused points at turn boundaries. Unlike ceil(path
/// cost / moves), this handles 1 -> 2 cost steps and the full-moves expensive-step rule.
fn travel_field(view: &PlayerView, unit: &UnitView, civilian: bool, limit: i32) -> Vec<i32> {
    let mut dist = vec![i32::MAX; view.tiles.len()];
    let Some(start) = view.index(unit.pos) else {
        return dist;
    };
    let moves = unit.max_moves.max(1);
    dist[start] = 0;
    let mut open = BinaryHeap::from([Reverse((0, start))]);
    let mut blocked = vec![false; view.tiles.len()];
    let mut neutral = vec![false; view.tiles.len()];
    for (owner, pos) in view
        .units
        .iter()
        .map(|u| (u.owner, u.pos))
        .chain(view.cities.iter().map(|c| (c.owner, c.pos)))
        .filter(|(owner, _)| !view.same_team(unit.owner, *owner))
    {
        if let Some(i) = view.index(pos) {
            blocked[i] = true;
            neutral[i] |= !view.at_war(unit.owner, owner);
        }
    }
    // Match the mover's border permissions, including mandatory progress toward an
    // exit for units stranded by a treaty. Only known owners/terrain constrain advice.
    let enterable = |h| {
        view.tile(h)
            .is_some_and(|t| view.passage_allowed(unit.owner, unit.kind, t.owner))
    };
    let egress = (!enterable(unit.pos)).then(|| {
        let mut map = crate::state::Map::new(view.width, view.height, Terrain::Grassland);
        for (i, tile) in view.tiles.iter().enumerate() {
            map.tiles[i].terrain = tile.terrain.unwrap_or(Terrain::Grassland);
        }
        let exits = view.exit_distances(&map, enterable);
        (map, exits)
    });
    while let Some(Reverse((d, i))) = open.pop() {
        if d != dist[i] || (i != start && blocked[i]) {
            continue;
        }
        for h in hex_at(view, i).neighbors() {
            let Some(j) = view.index(h) else { continue };
            let t = &view.tiles[j];
            let Some(cost) = Some(t).filter(|t| t.explored).and_then(|t| t.move_cost()) else {
                continue;
            };
            let border_allowed = egress.as_ref().map_or_else(
                || enterable(h),
                |(map, exits)| {
                    crate::path::border_step(map, hex_at(view, i), h, enterable, Some(exits))
                },
            );
            if !border_allowed || neutral[j] || (civilian && blocked[j]) {
                continue;
            }
            let cost = cost.min(moves);
            let remaining = moves - d % moves;
            let next = d + if cost > remaining {
                remaining + cost
            } else {
                cost
            };
            if next <= limit && next < dist[j] {
                dist[j] = next;
                open.push(Reverse((next, j)));
            }
        }
    }
    dist
}

/// Potential attack reach next turn, only from known enemy military. Ranged orders fire
/// before movement, so range is measured from the current tile, never from every move endpoint.
pub fn known_threats(view: &PlayerView) -> Vec<u32> {
    let mut threats = vec![0; view.tiles.len()];
    for unit in view
        .units
        .iter()
        .filter(|u| u.military && view.is_threat(view.me, u.owner))
    {
        let reach = travel_field(view, unit, false, unit.max_moves);
        for (i, count) in threats.iter_mut().enumerate() {
            if view.tiles[i].explored
                && (reach[i] != i32::MAX
                    || (unit.ranged > 0 && unit.pos.distance(hex_at(view, i)) <= unit.range))
            {
                *count += 1;
            }
        }
    }
    threats
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScorePart {
    pub label: String,
    /// Additive change after integer rounding. These always sum to the displayed score.
    pub value: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkedTile {
    pub pos: Hex,
    pub terrain: Terrain,
    pub yields: Yields,
}

/// A known resource a city here could work or claim: within the work radius, not foreign.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteResource {
    pub resource: Resource,
    pub distance: i32,
}

/// A natural wonder no one owns within the work radius, with the points it adds to the quality.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteWonder {
    pub wonder: Wonder,
    pub distance: i32,
    pub value: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementSite {
    pub pos: Hex,
    pub score: i32,
    /// The absolute value of the site (`site_quality`) and its band.
    pub quality: i32,
    pub band: SiteBand,
    /// A visible foreign border makes an unseen city within spacing range possible.
    pub border_risk: bool,
    /// What the quality is made of, for the hover (explained sites only).
    pub resources: Vec<SiteResource>,
    pub wonders: Vec<SiteWonder>,
    pub breakdown: Vec<ScorePart>,
    pub potential_yields: Yields,
    /// Terrain yields only; city/faction/building income is not a tile yield.
    pub center_yields: Yields,
    /// Ring-one tiles a city founded here would own and work from the start (population is
    /// territory: all of them, in hex order). Foreign-owned and unexplored tiles are left out.
    pub ring_one: Vec<WorkedTile>,
    pub terrain_bonus: Yields,
    pub defense_bonus: i32,
    pub nearest_city: Option<i32>,
    pub foreign_tiles: i32,
    pub own_border_tiles: i32,
    pub unknown_tiles: i32,
    pub travel_turns: Option<i32>,
    pub threats: u32,
    pub rank: Option<u8>,
}

/// Work-radius potential, also used when the AI decides whether to build another settler.
/// Population is territory: a city owns and works ring one from the start, claims ring two as
/// it grows to about population 7 and ring three after that, so ring-three tiles count at half
/// weight (each yield rounded down). Foreign borders lose their yields and another eight
/// points; existing own borders are reported separately (founding can reassign the initial
/// ring). Resources the viewer knows (the view already hides unrevealed ones) add their tile
/// bonus.
pub fn site_potential(view: &PlayerView, pos: Hex) -> (Yields, i32) {
    let mut yields = Yields::ZERO;
    let mut foreign = 0;
    for h in pos.within_iter(CITY_WORK_RADIUS) {
        let Some(t) = view.tile(h).filter(|t| t.explored) else {
            continue;
        };
        if t.owner.is_some_and(|o| o != view.me) {
            foreign += 1;
            continue;
        }
        if let Some(terrain) = t.terrain.filter(|t| t.is_land()) {
            let tile = site_tile_yields(view, view.faction_of(view.me), terrain, t.resource);
            yields += if h.distance(pos) >= CITY_WORK_RADIUS {
                Yields::new(
                    tile.food / 2,
                    tile.production / 2,
                    tile.gold / 2,
                    tile.science / 2,
                )
            } else {
                tile
            };
        }
    }
    (yields, foreign)
}

/// One land tile's part in a site's potential: the faction's terrain yields plus a known
/// resource's bonus. Under the Nile (`economy::nile`) Egypt's Food is doubled and it has no
/// Production, so an Egyptian Settler values land by its Food (designer, 2026-11-22: Egypt
/// doesn't see Production).
fn site_tile_yields(
    view: &PlayerView,
    faction: Faction,
    terrain: Terrain,
    resource: Option<resources::Resource>,
) -> Yields {
    let y = data::terrain_yields(faction, terrain)
        + resource.map_or(Yields::ZERO, |r| resources::stats(r).bonus);
    if faction == Faction::Egypt && view.rule_experiments.egypt_nile() {
        let pct = view.rule_experiments.egypt_nile_food_percent();
        Yields::new(y.food * pct / 100, 0, y.gold, y.science)
    } else {
        y
    }
}

/// Known strategic deposits a city here could claim: within the work radius, not foreign.
pub fn site_strategic(view: &PlayerView, pos: Hex) -> i32 {
    pos.within_iter(CITY_WORK_RADIUS)
        .filter_map(|h| view.tile(h))
        .filter(|t| {
            t.owner.is_none_or(|o| o == view.me) && t.resource.is_some_and(resources::is_strategic)
        })
        .count() as i32
}

/// A nearby unclaimed landmark is a compounding city asset; ring one is claimed at founding.
pub fn site_wonders(view: &PlayerView, pos: Hex) -> i32 {
    pos.within_iter(CITY_WORK_RADIUS)
        .filter_map(|h| view.tile(h).map(|t| (h, t)))
        .filter(|(_, t)| t.wonder.is_some() && t.owner.is_none())
        .map(|(h, _)| match h.distance(pos) {
            0 | 1 => 80,
            2 => 50,
            _ => 25,
        })
        .sum()
}

/// The unowned natural wonders within the work radius, nearest first, each with the points
/// `site_wonders` gives it.
pub fn site_wonders_near(view: &PlayerView, pos: Hex) -> Vec<SiteWonder> {
    let mut found: Vec<SiteWonder> = pos
        .within_iter(CITY_WORK_RADIUS)
        .filter_map(|h| view.tile(h).map(|t| (h, t)))
        .filter_map(|(h, t)| {
            let wonder = t.wonder.filter(|_| t.owner.is_none())?;
            let distance = h.distance(pos);
            let value = match distance {
                0 | 1 => 80,
                2 => 50,
                _ => 25,
            };
            Some(SiteWonder {
                wonder,
                distance,
                value,
            })
        })
        .collect();
    found.sort_by_key(|w| (w.distance, w.wonder));
    found
}

/// Known resources within the work radius that a city here could claim (not foreign-owned),
/// nearest first. Only the viewer's known resources appear: a hidden strategic deposit does not.
pub fn site_resources(view: &PlayerView, pos: Hex) -> Vec<SiteResource> {
    let mut found: Vec<SiteResource> = pos
        .within_iter(CITY_WORK_RADIUS)
        .filter_map(|h| view.tile(h).map(|t| (h, t)))
        .filter(|(_, t)| t.explored && t.owner.is_none_or(|o| o == view.me))
        .filter_map(|(h, t)| {
            Some(SiteResource {
                resource: t.resource?,
                distance: h.distance(pos),
            })
        })
        .collect();
    found.sort_by_key(|r| (r.distance, r.resource));
    found
}

pub fn base_site_score(view: &PlayerView, pos: Hex) -> i32 {
    let (yields, foreign) = site_potential(view, pos);
    tile_score(yields, CityFocus::Balanced) - foreign * 8
        + site_strategic(view, pos) * resources::STRATEGIC_SITE_VALUE
        + site_wonders(view, pos)
}

/// The viewer-level settlement facts every settler's analysis and the planner's site scores
/// share, worked out once per view (each on first use): border risks, known enemy reach and,
/// per tile, whether it's a site and its potential. Use one per view; a view edited after
/// first use needs a new one.
pub struct SiteBasics {
    border_risks: Vec<bool>,
    /// What each tile adds to a site's potential and strategic count, per tile index.
    inputs: std::cell::OnceCell<Vec<TileInput>>,
    /// The `SiteMemo` generation this view last synced with (0 = not yet).
    synced: std::cell::Cell<u64>,
    /// Tiles within the viewer's founding distance of a known city (`legal_site`'s spacing rule).
    near_city: std::cell::OnceCell<Vec<bool>>,
    threats: std::cell::OnceCell<Vec<u32>>,
    facts: Vec<std::cell::OnceCell<Option<SiteFacts>>>,
}

/// One tile's part in `site_potential`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Contribution {
    /// Unexplored, or explored water or unknown terrain: nothing.
    None,
    /// Explored and foreign-owned: counted, yields excluded.
    Foreign,
    /// Explored own or unowned land: its yields with known resources.
    Land(Yields),
}

#[derive(Clone, Copy)]
struct SiteFacts {
    /// A foreign border makes an unseen city within spacing range possible (`border_city_risks`).
    risk: bool,
    potential: Yields,
    foreign: i32,
    strategic: i32,
}

/// Everything a tile adds to the sites around it: its `site_potential` contribution and
/// whether it counts for `site_strategic` (a known strategic deposit, not foreign).
#[derive(Clone, Copy, PartialEq, Eq)]
struct TileInput {
    contribution: Contribution,
    strategic: bool,
}

/// A site's potential, foreign tile count and strategic count: a function of the
/// `TileInput`s within `CITY_WORK_RADIUS` of it and nothing else.
type SiteSums = (Yields, i32, i32);

/// Site sums carried from one view to the next (across turns, seats' planners and search
/// rollouts on this thread). Each entry holds the tile inputs its sums were computed from;
/// a view syncs by comparing its inputs tile by tile and forgetting the sums of every site
/// within `CITY_WORK_RADIUS` of a changed tile. Exact for any view, so entries are kept per
/// viewer only so seats don't evict each other.
struct SiteMemo {
    viewer: PlayerId,
    width: i32,
    height: i32,
    inputs: Vec<TileInput>,
    sums: Vec<Option<SiteSums>>,
    /// Bumped by every sync; a view whose generation is stale syncs again before reading.
    generation: u64,
}

thread_local! {
    static SITE_MEMOS: std::cell::RefCell<Vec<SiteMemo>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Memo entries kept per thread (one per recent viewer).
const SITE_MEMO_VIEWERS: usize = 9;
static SITE_MEMO_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl SiteBasics {
    pub fn new(view: &PlayerView) -> Self {
        Self {
            border_risks: border_city_risks(view),
            inputs: std::cell::OnceCell::new(),
            synced: std::cell::Cell::new(0),
            near_city: std::cell::OnceCell::new(),
            threats: std::cell::OnceCell::new(),
            facts: (0..view.tiles.len())
                .map(|_| std::cell::OnceCell::new())
                .collect(),
        }
    }

    fn threats(&self, view: &PlayerView) -> &[u32] {
        self.threats.get_or_init(|| known_threats(view))
    }

    /// Tile `i` when it's a known legal site clear of border risk (else None): the sites the
    /// planner may send a Settler to.
    fn facts(&self, view: &PlayerView, i: usize) -> Option<SiteFacts> {
        self.legal_facts(view, i).filter(|f| !f.risk)
    }

    /// Tile `i` when it's a known legal site, flagged when a foreign border makes an unseen
    /// city nearby possible. The map shows these too (a risky site is still legal).
    fn legal_facts(&self, view: &PlayerView, i: usize) -> Option<SiteFacts> {
        *self.facts[i].get_or_init(|| {
            let pos = hex_at(view, i);
            if !self.known_legal_site(view, i, pos) {
                return None;
            }
            let (potential, foreign, strategic) = self.site_sums(view, i, pos);
            Some(SiteFacts {
                risk: self.border_risks[i],
                potential,
                foreign,
                strategic,
            })
        })
    }

    /// `known_legal_site(view, pos)` for tile `i`, with city spacing from a precomputed mask.
    fn known_legal_site(&self, view: &PlayerView, i: usize, pos: Hex) -> bool {
        let near_city = self.near_city.get_or_init(|| {
            let mut near = vec![false; view.tiles.len()];
            for c in view.cities.iter() {
                for h in c
                    .pos
                    .within_iter(view.rule_experiments.min_city_gap(view.faction_of(view.me)))
                {
                    if let Some(j) = view.index(h) {
                        near[j] = true;
                    }
                }
            }
            near
        });
        let t = &view.tiles[i];
        let legal = t.explored
            && t.wonder.is_none()
            && t.terrain.is_some_and(|terrain| {
                legal_site(
                    view.me,
                    view.rule_experiments.min_city_gap(view.faction_of(view.me)),
                    terrain,
                    t.owner,
                    pos,
                    std::iter::empty(),
                )
            })
            && !near_city[i]
            && !view.city_gap_blocked.contains(&pos)
            && !view.failed_city_sites.contains(&pos);
        debug_assert_eq!(legal, known_legal_site(view, pos), "cached legal site");
        legal
    }

    /// Each tile's `TileInput` for this view.
    fn inputs(&self, view: &PlayerView) -> &[TileInput] {
        self.inputs.get_or_init(|| {
            let faction = view.faction_of(view.me);
            view.tiles
                .iter()
                .map(|t| TileInput {
                    contribution: if !t.explored {
                        Contribution::None
                    } else if t.owner.is_some_and(|o| o != view.me) {
                        Contribution::Foreign
                    } else if let Some(terrain) = t.terrain.filter(|t| t.is_land()) {
                        Contribution::Land(site_tile_yields(view, faction, terrain, t.resource))
                    } else {
                        Contribution::None
                    },
                    strategic: t.owner.is_none_or(|o| o == view.me)
                        && t.resource.is_some_and(resources::is_strategic),
                })
                .collect()
        })
    }

    /// Bring this thread's memo entry for the viewer in line with this view's inputs:
    /// forget the sums of every site within `CITY_WORK_RADIUS` of a tile whose input changed.
    fn sync<'m>(&self, view: &PlayerView, memos: &'m mut Vec<SiteMemo>) -> &'m mut SiteMemo {
        let inputs = self.inputs(view);
        let k = match memos.iter().position(|m| m.viewer == view.me) {
            Some(k) => k,
            None => {
                if memos.len() >= SITE_MEMO_VIEWERS {
                    memos.remove(0);
                }
                memos.push(SiteMemo {
                    viewer: view.me,
                    width: 0,
                    height: 0,
                    inputs: Vec::new(),
                    sums: Vec::new(),
                    // Never a view's generation: the first read syncs.
                    generation: u64::MAX,
                });
                memos.len() - 1
            }
        };
        let memo = &mut memos[k];
        if memo.generation != self.synced.get() {
            if memo.width != view.width
                || memo.height != view.height
                || memo.inputs.len() != inputs.len()
            {
                memo.width = view.width;
                memo.height = view.height;
                memo.inputs = inputs.to_vec();
                memo.sums = vec![None; inputs.len()];
            } else {
                for (i, (old, new)) in memo.inputs.iter_mut().zip(inputs).enumerate() {
                    if old != new {
                        *old = *new;
                        for h in hex_at(view, i).within_iter(CITY_WORK_RADIUS) {
                            if let Some(j) = view.index(h) {
                                memo.sums[j] = None;
                            }
                        }
                    }
                }
            }
            memo.generation =
                SITE_MEMO_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.synced.set(memo.generation);
        }
        memo
    }

    /// `(site_potential(view, pos), site_strategic(view, pos))` for tile `i`, reused from
    /// earlier views while every tile input within `CITY_WORK_RADIUS` is unchanged.
    fn site_sums(&self, view: &PlayerView, i: usize, pos: Hex) -> SiteSums {
        let sums = SITE_MEMOS.with(|memos| {
            let mut memos = memos.borrow_mut();
            let memo = self.sync(view, &mut memos);
            if let Some(sums) = memo.sums[i] {
                return sums;
            }
            let sums = Self::sum_inputs(view, &memo.inputs, pos);
            memo.sums[i] = Some(sums);
            sums
        });
        debug_assert_eq!(
            sums,
            {
                let (potential, foreign) = site_potential(view, pos);
                (potential, foreign, site_strategic(view, pos))
            },
            "cached site sums at {pos:?}"
        );
        sums
    }

    fn sum_inputs(view: &PlayerView, inputs: &[TileInput], pos: Hex) -> SiteSums {
        let mut yields = Yields::ZERO;
        let mut foreign = 0;
        let mut strategic = 0;
        for h in pos.within_iter(CITY_WORK_RADIUS) {
            let Some(i) = view.index(h) else { continue };
            let input = inputs[i];
            strategic += i32::from(input.strategic);
            match input.contribution {
                Contribution::None => {}
                Contribution::Foreign => foreign += 1,
                Contribution::Land(tile) => {
                    yields += if h.distance(pos) >= CITY_WORK_RADIUS {
                        Yields::new(
                            tile.food / 2,
                            tile.production / 2,
                            tile.gold / 2,
                            tile.science / 2,
                        )
                    } else {
                        tile
                    };
                }
            }
        }
        (yields, foreign, strategic)
    }

    /// `base_site_score` of tile `i` when it's a known legal site clear of border risk.
    pub fn base_score(&self, view: &PlayerView, i: usize) -> Option<i32> {
        self.facts(view, i).map(|f| Self::score_of(view, i, f))
    }

    /// `base_score`, also for a legal site a foreign border puts at risk.
    pub fn legal_base_score(&self, view: &PlayerView, i: usize) -> Option<i32> {
        self.legal_facts(view, i)
            .map(|f| Self::score_of(view, i, f))
    }

    fn score_of(view: &PlayerView, i: usize, f: SiteFacts) -> i32 {
        tile_score(f.potential, CityFocus::Balanced) - f.foreign * 8
            + f.strategic * resources::STRATEGIC_SITE_VALUE
            + site_wonders(view, hex_at(view, i))
    }
}

/// Score all known legal sites. `reserved` is the planner's already chosen settlement sites;
/// it affects the Mongols network preference, not founding legality. Human clients supply
/// other settlers' ordered destinations. Travel is bounded only for top-three markers.
pub fn settlement_sites(
    view: &PlayerView,
    unit: &UnitView,
    reserved: &[Hex],
) -> Vec<SettlementSite> {
    let basics = SiteBasics::new(view);
    let mut sites = analyze_sites(view, &basics, unit, 0..view.tiles.len(), true, true);
    rank_sites(view, reserved, &mut sites);
    sites
}

/// Numeric planning facts only: no explanation strings, worked-tile detail or UI ranking.
pub struct SettlementCandidate {
    pub pos: Hex,
    pub score: i32,
    pub nearest_city: Option<i32>,
    pub travel_turns: Option<i32>,
    pub threats: u32,
    /// A visible foreign border could hide a city within spacing (`border_city_risks`). Only
    /// `settlement_candidates_with_risky` returns such sites.
    pub border_risk: bool,
}

pub fn settlement_candidates(
    view: &PlayerView,
    unit: &UnitView,
    reserved: &[Hex],
) -> Vec<SettlementCandidate> {
    settlement_candidates_with(view, &SiteBasics::new(view), unit, reserved)
}

/// `settlement_candidates` sharing `basics` (`SiteBasics::new(view)`) across settlers.
pub fn settlement_candidates_with(
    view: &PlayerView,
    basics: &SiteBasics,
    unit: &UnitView,
    _reserved: &[Hex],
) -> Vec<SettlementCandidate> {
    candidates(view, basics, unit, false)
}

/// `settlement_candidates_with`, also listing the legal sites a foreign border puts at risk
/// (flagged `border_risk`), for a planner that weighs that evidence itself.
pub fn settlement_candidates_with_risky(
    view: &PlayerView,
    basics: &SiteBasics,
    unit: &UnitView,
) -> Vec<SettlementCandidate> {
    candidates(view, basics, unit, true)
}

fn candidates(
    view: &PlayerView,
    basics: &SiteBasics,
    unit: &UnitView,
    include_risky: bool,
) -> Vec<SettlementCandidate> {
    analyze_sites(
        view,
        basics,
        unit,
        0..view.tiles.len(),
        false,
        include_risky,
    )
    .into_iter()
    .map(|s| SettlementCandidate {
        pos: s.pos,
        score: s.score,
        nearest_city: s.nearest_city,
        travel_turns: s.travel_turns,
        threats: s.threats,
        border_risk: s.border_risk,
    })
    .collect()
}

/// The top three recommendable sites: highest band first, so a marker never sits on a lower band
/// than a later one, then the travel-adjusted score (nearer and safer first), then tile order.
fn rank_sites(view: &PlayerView, reserved: &[Hex], sites: &mut [SettlementSite]) {
    let mut ranked: Vec<_> = sites
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            s.score > 0
                && !s.border_risk
                && s.threats == 0
                && s.travel_turns.is_some_and(|t| t <= RECOMMEND_TRAVEL_TURNS)
                && !reserved.iter().any(|h| {
                    h.distance(s.pos)
                        <= view.rule_experiments.min_city_gap(view.faction_of(view.me))
                })
                && !view.units.iter().any(|u| {
                    u.owner != view.me
                        && u.kind == UnitKind::Settler
                        && u.pos.distance(s.pos)
                            <= view.rule_experiments.min_city_gap(view.faction_of(u.owner))
                })
        })
        .map(|(i, s)| (Reverse((s.band, s.score)), i))
        .collect();
    ranked.sort();
    for (rank, (_, i)) in ranked.into_iter().take(3).enumerate() {
        sites[i].rank = Some(rank as u8 + 1);
    }
}

/// The map needs scores, reachability and ranks, not an explanation per tile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementScore {
    pub pos: Hex,
    pub score: i32,
    /// Absolute value and band: the same for any Settler, anywhere, on any map.
    pub quality: i32,
    pub band: SiteBand,
    pub border_risk: bool,
    pub rank: Option<u8>,
    pub travel_turns: Option<i32>,
}

pub fn settlement_scores(
    view: &PlayerView,
    unit: &UnitView,
    reserved: &[Hex],
) -> Vec<SettlementScore> {
    let basics = SiteBasics::new(view);
    let mut sites = analyze_sites(view, &basics, unit, 0..view.tiles.len(), false, true);
    rank_sites(view, reserved, &mut sites);
    sites
        .into_iter()
        .map(|s| SettlementScore {
            pos: s.pos,
            score: s.score,
            quality: s.quality,
            band: s.band,
            border_risk: s.border_risk,
            rank: s.rank,
            travel_turns: s.travel_turns,
        })
        .collect()
}

/// Explain exactly one hex. The caller already has its rank in the compact map payload.
pub fn settlement_site(
    view: &PlayerView,
    unit: &UnitView,
    _reserved: &[Hex],
    pos: Hex,
) -> Option<SettlementSite> {
    analyze_sites(
        view,
        &SiteBasics::new(view),
        unit,
        view.index(pos),
        true,
        true,
    )
    .pop()
}

fn analyze_sites(
    view: &PlayerView,
    basics: &SiteBasics,
    unit: &UnitView,
    indices: impl IntoIterator<Item = usize>,
    explain: bool,
    // The map lists every legal site; the planner leaves out those a foreign border puts at risk.
    include_risky: bool,
) -> Vec<SettlementSite> {
    let faction = view.faction_of(view.me);
    let roman_founder = faction == Faction::Rome
        && !view
            .rule_experiments
            .rule_off(FactionRule::RomeVeteranCities)
        && data::unit_class(unit.kind) == Some(crate::UnitClass::Melee)
        && unit.level >= 1
        && unit.hp == data::UNIT_HP;
    if unit.owner != view.me || (unit.kind != UnitKind::Settler && !roman_founder) {
        return Vec::new();
    }
    let own: Vec<_> = view.cities.iter().filter(|c| c.owner == view.me).collect();
    let capital = own.iter().find(|c| c.is_capital);
    let travel = travel_field(view, unit, true, i32::MAX);
    let threats = basics.threats(view);
    let moves = unit.max_moves.max(1);
    indices
        .into_iter()
        .filter_map(|i| {
            let pos = hex_at(view, i);
            let facts = if include_risky {
                basics.legal_facts(view, i)?
            } else {
                basics.facts(view, i)?
            };
            let terrain = view.tiles[i].terrain.expect("legal terrain");
            let nearest_city = own.iter().map(|c| c.pos.distance(pos)).min();
            let turns = (travel[i] != i32::MAX).then(|| (travel[i] + moves - 1) / moves);
            let (potential_yields, foreign_tiles) = (facts.potential, facts.foreign);
            let initial = tile_score(potential_yields, CityFocus::Balanced);
            let mut score = initial;
            let mut breakdown = Vec::new();
            if explain {
                let nile = faction == Faction::Egypt && view.rule_experiments.egypt_nile();
                breakdown.push(ScorePart {
                    label: if nile {
                        "Land within 3 tiles, outer ring at half: Food ×4 + Gold ×2 (Food doubled by the Nile)"
                    } else {
                        "Land within 3 tiles, outer ring at half: Food ×4 + Production ×3 + Gold ×2"
                    }
                    .into(),
                    value: score,
                });
            }
            let mut adjust = |label: &dyn Fn() -> String, next: i32| {
                if explain {
                    breakdown.push(ScorePart {
                        label: label(),
                        value: next - score,
                    });
                }
                score = next;
                next
            };
            let mut value = adjust(
                &|| "Foreign borders: −8 per tile (their yields excluded)".into(),
                initial - foreign_tiles * 8,
            );
            let strategic = facts.strategic;
            if strategic > 0 {
                value = adjust(
                    &|| {
                        format!(
                            "Known strategic deposits within 3: +{} × {strategic}",
                            resources::STRATEGIC_SITE_VALUE
                        )
                    },
                    value + strategic * resources::STRATEGIC_SITE_VALUE,
                );
            }
            let wonders = site_wonders(view, pos);
            if wonders > 0 {
                value = adjust(&|| "Natural wonder within reach".into(), value + wonders);
            }
            // Everything above is about the land alone: the absolute quality.
            let quality = value;
            if let Some(turns) = turns {
                let rate = if own.is_empty() {
                    if travel[i] > 2 { 60 } else { 25 }
                } else {
                    40
                };
                value = adjust(
                    &|| format!("Travel: ×100 / (100 + {rate} × {turns} turns)"),
                    value * 100 / (100 + rate * turns),
                );
            }
            if let Some(home) = nearest_city {
                if home > 6 && faction != Faction::Mongols {
                    value = adjust(
                        &|| format!("Distance: ×100 / (100 + 15 × {} hexes beyond 6)", home - 6),
                        value * 100 / (100 + 15 * (home - 6)),
                    );
                }
                if faction == Faction::India {
                    let linked = own.iter().filter(|c| c.pos.distance(pos) <= INDIA_NETWORK_RADIUS).count() as i32;
                    if linked > 0 && !view.rule_experiments.rule_off(FactionRule::IndiaNetwork) {
                        let percent = linked * INDIA_NETWORK_PERCENT_PER_CITY;
                        let food = (potential_yields.food * percent + 50) / 100;
                        let production = (potential_yields.production * percent + 50) / 100;
                        let bonus = if initial > 0 { value * (food * 4 + production * 3) / initial } else { 0 };
                        value = adjust(
                            &|| format!("India local network: {linked} linked cities, +{}% Food/Production", linked * INDIA_NETWORK_PERCENT_PER_CITY),
                            value + bonus,
                        );
                    }
                    if let Some(cap) = capital.filter(|_| !view.rule_experiments.rule_off(FactionRule::IndiaCorruption)) {
                        let beyond = (cap.pos.distance(pos) - INDIA_CAPITAL_RADIUS).max(0);
                        if beyond > 0 {
                            let loss = india_distance_loss_percent(beyond);
                            value = adjust(
                                &|| format!("Corruption: {} hexes from the capital (-{loss}%)", cap.pos.distance(pos)),
                                value * (100 - loss) / 100,
                            );
                        }
                    }
                }            }
            let mut unknown_tiles = 0;
            let mut own_border_tiles = 0;
            let mut terrain_bonus = Yields::ZERO;
            let mut worked = Vec::new();
            for h in pos
                .within(CITY_WORK_RADIUS)
                .into_iter()
                .filter(|_| explain)
            {
                let Some(t) = view.tile(h) else { continue };
                if !t.explored {
                    unknown_tiles += 1;
                    continue;
                }
                if t.owner == Some(view.me) {
                    own_border_tiles += 1;
                }
                let Some(terrain) = t.terrain else { continue };
                if t.owner.is_some_and(|o| o != view.me) {
                    continue;
                }
                if !explain {
                    continue;
                }
                terrain_bonus += data::terrain_yields(faction, terrain) - terrain.yields();
                let yields = site_tile_yields(view, faction, terrain, None);
                if h != pos && h.distance(pos) <= 1 {
                    worked.push(WorkedTile {
                        pos: h,
                        terrain,
                        yields,
                    });
                }
            }
            if pos == unit.pos {
                value = adjust(&|| "Found without moving: ×110%".into(), value * 110 / 100);
            }
            if unit.goto == Some(pos) {
                value = adjust(
                    &|| "Continue chosen destination: ×130%".into(),
                    value * 130 / 100,
                );
            }
            if threats[i] > 0 {
                adjust(
                    &|| format!("Known enemy reach: −60 × {} units", threats[i]),
                    value - 60 * threats[i] as i32,
                );
            }
            worked.sort_by_key(|t| t.pos);
            Some(SettlementSite {
                pos,
                score,
                quality,
                band: SiteBand::of(quality),
                border_risk: facts.risk,
                resources: if explain { site_resources(view, pos) } else { Vec::new() },
                wonders: if explain { site_wonders_near(view, pos) } else { Vec::new() },
                breakdown,
                potential_yields,
                center_yields: site_tile_yields(view, faction, terrain, None),
                ring_one: worked,
                terrain_bonus,
                defense_bonus: data::terrain_defense_rules(
                    view.rule_experiments.faction_rules_off,
                    faction,
                    terrain,
                ),
                nearest_city,
                foreign_tiles,
                own_border_tiles,
                unknown_tiles,
                travel_turns: turns,
                threats: threats[i],
                rank: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;

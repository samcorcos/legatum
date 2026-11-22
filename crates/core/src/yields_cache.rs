//! Cross-turn cache of city yield computations for view building (`view::player_view_cached`).
//!
//! A city's yields read only a few things: its position, owner, population, resistance,
//! buildings and which of them are idle for lack of supply; its owner's decadence, faction,
//! technologies and post-research Science conversion; the tiles within the work radius;
//! barbarians holding any of them; and where
//! its owner's other cities and capital are (capital bonus, India network and distance,
//! Mongols commerce); and every wonder in the city's territory, including beyond work radius.
//! `CityKey` holds exactly those inputs, compared by value, so an entry is reused only when
//! every input is unchanged, from any state: a cache can be shared across
//! turns, seats and even unrelated states (search rollouts) without ever going stale.
//! Rule experiments are compared once per cache (`ViewCache::begin`).
//!
//! Debug builds recompute every cached read from scratch and assert equality, so a missing
//! input fails loudly in tests instead of drifting silently.
use crate::data::CITY_WORK_RADIUS;
use crate::economy;
use crate::hex::Hex;
use crate::resources::Resource;
use crate::state::{City, GameState};
use crate::types::{
    BuildingKind, CityId, Faction, PlayerId, ProductionItem, ScienceConversion, TechId, Terrain,
    Yields,
};
use crate::wonders::Wonder;
use std::collections::{BTreeMap, BTreeSet};

/// City yield results carried from one view build to the next. Hold one per game (or per
/// rollout); it never changes results, only how often they are computed.
#[derive(Debug, Default)]
pub struct ViewCache {
    rules: Option<crate::experiments::RuleExperiments>,
    cities: BTreeMap<CityId, CityCache>,
    /// Counts view builds: within one build the state is fixed, so an entry checked against
    /// it once (`CityCache::checked`) needs no second check.
    build: u64,
}

/// Everything a city's yields read (see the module docs).
#[derive(Clone, Debug, PartialEq, Eq)]
struct CityKey {
    owner: PlayerId,
    faction: Faction,
    pos: Hex,
    population: i32,
    resistance: i32,
    /// The civ whose war freezes the city's Production (`rules::occupation::city_freeze`).
    freeze: Option<PlayerId>,
    /// The owner's decadence, the cut taken from every yield when enabled.
    decadence: i32,
    buildings: BTreeSet<BuildingKind>,
    /// Building tiers idle for lack of strategic supply (`resources::idle_tiers`).
    idle: Vec<(BuildingKind, Resource)>,
    techs: BTreeSet<TechId>,
    science_conversion: ScienceConversion,
    /// Tiles within the work radius, in `Hex::within_iter` order: what yields read of the
    /// centre and of the city's own tiles (`tile_key`); None for every other tile.
    tiles: Vec<TileKey>,
    wonders: Vec<(Hex, Wonder)>,
    squatted: Vec<Hex>,
    /// The owner's cities (id, position, capital), in id order.
    empire: Vec<(CityId, Hex, bool)>,
}

/// Terrain, resource and improvement of a tile a city's yields read: its centre and the tiles
/// it owns (`Tile::city`). The rest of the radius doesn't enter them (barbarian-held tiles are separate).
type TileKey = Option<(Terrain, Option<Resource>, bool)>;

fn city_wonders(state: &GameState, city: &City) -> Vec<(Hex, Wonder)> {
    state
        .map
        .tiles
        .iter()
        .enumerate()
        .filter_map(|(i, tile)| {
            (tile.city == Some(city.id))
                .then_some((state.map.hex_at(i), tile.wonder))
                .and_then(|(h, wonder)| wonder.map(|w| (h, w)))
        })
        .collect()
}

fn tile_key(state: &GameState, city: &City, h: Hex) -> TileKey {
    state
        .map
        .get(h)
        .filter(|t| h == city.pos || t.city == Some(city.id))
        .map(|t| (t.terrain, t.resource, t.improved))
}

impl CityKey {
    /// Whether this key still describes `city` in `state`, without building a new key.
    fn matches(&self, state: &GameState, city: &City) -> bool {
        let owner = state.player(city.owner);
        self.owner == city.owner
            && self.faction == owner.faction
            && self.pos == city.pos
            && self.population == city.population
            && self.resistance == city.resistance
            && self.freeze == crate::rules::occupation::city_freeze(state, city)
            && self.decadence == crate::scoring::decadence_percent(state, owner.id)
            && self.buildings == city.buildings
            && self.idle == crate::resources::idle_tiers(owner, city)
            && self.techs == owner.techs
            && self.science_conversion == owner.science_conversion
            && self.tiles.iter().copied().eq(city
                .pos
                .within_iter(CITY_WORK_RADIUS)
                .map(|h| tile_key(state, city, h)))
            && self.wonders == city_wonders(state, city)
            && self.empire.iter().copied().eq(state
                .cities_of(city.owner)
                .map(|c| (c.id, c.pos, c.is_capital)))
            && self.squatted == economy::denied_near(state, city)
    }

    fn of(state: &GameState, city: &City) -> Self {
        let owner = state.player(city.owner);
        Self {
            owner: city.owner,
            faction: owner.faction,
            pos: city.pos,
            population: city.population,
            resistance: city.resistance,
            freeze: crate::rules::occupation::city_freeze(state, city),
            decadence: crate::scoring::decadence_percent(state, owner.id),
            buildings: city.buildings.clone(),
            idle: crate::resources::idle_tiers(owner, city),
            techs: owner.techs.clone(),
            science_conversion: owner.science_conversion,
            tiles: city
                .pos
                .within_iter(CITY_WORK_RADIUS)
                .map(|h| tile_key(state, city, h))
                .collect(),
            wonders: city_wonders(state, city),
            squatted: economy::denied_near(state, city),
            empire: state
                .cities_of(city.owner)
                .map(|c| (c.id, c.pos, c.is_capital))
                .collect(),
        }
    }
}

/// (population, resistance > 0, the build when it is an industry project).
type TotalsSlot = (i32, bool, Option<ProductionItem>, bool);

/// One city's cached results, valid while its `CityKey` holds.
#[derive(Debug)]
pub(crate) struct CityCache {
    key: CityKey,
    /// The view build (`ViewCache::build`) that last checked `key` against the state.
    checked: u64,
    /// `city_totals_squatted` by (population, resistance > 0, project item).
    totals: Vec<(TotalsSlot, Yields)>,
    /// `city_yields_mode(.., None, None, ..)` with nothing or one building installed, by
    /// (building, full).
    yields: Vec<((Option<BuildingKind>, bool), economy::CityYields)>,
    /// `city_yields_base_mode` by `full`: the build-independent part the view derives every
    /// build's yields from.
    base: Vec<(bool, economy::CityYields)>,
}

impl ViewCache {
    /// Call at the start of each view build: forgets everything if the rules differ, and
    /// drops cities that no longer exist.
    pub(crate) fn begin(&mut self, state: &GameState) {
        self.build += 1;
        if self.rules != Some(state.setup.rule_experiments) {
            self.rules = Some(state.setup.rule_experiments);
            self.cities.clear();
        }
        self.cities.retain(|id, _| state.cities.contains_key(id));
    }

    /// The entry for `city`, taken out of the cache (fresh if any input changed). Return it
    /// with `put` so later builds can reuse it.
    pub(crate) fn take(&mut self, state: &GameState, city: &City) -> CityCache {
        let build = self.build;
        match self.cities.remove(&city.id) {
            Some(entry) if entry.checked == build => {
                debug_assert_eq!(
                    entry.key,
                    CityKey::of(state, city),
                    "state changed in a build"
                );
                entry
            }
            Some(mut entry) if entry.key.matches(state, city) => {
                debug_assert_eq!(entry.key, CityKey::of(state, city), "CityKey::matches");
                entry.checked = build;
                entry
            }
            _ => CityCache {
                key: CityKey::of(state, city),
                checked: build,
                totals: Vec::new(),
                yields: Vec::new(),
                base: Vec::new(),
            },
        }
    }

    pub(crate) fn put(&mut self, city: CityId, entry: CityCache) {
        self.cities.insert(city, entry);
    }
}

impl CityCache {
    /// `economy::denied_near(state, city)` for the city this entry was taken for.
    pub(crate) fn squatted(&self) -> &[Hex] {
        &self.key.squatted
    }

    /// `city_totals_squatted(state, variant, item, None, squatted)` for `city` at
    /// `population` and `resistance` (growth forecasts walk those).
    pub(crate) fn totals(
        &mut self,
        state: &GameState,
        city: &City,
        population: i32,
        resistance: i32,
        item: Option<ProductionItem>,
    ) -> Yields {
        // Totals depend on the build only through projects (Germany's industry, Egypt's Grow)
        // and, on Egypt's Nile, whether anything else is being built.
        let project = item.filter(|i| matches!(i, ProductionItem::Project { .. }));
        let slot = (
            population,
            resistance > 0,
            project,
            economy::egypt_nile_builds(item),
        );
        let fresh = || {
            let mut variant = city.clone();
            variant.population = population;
            variant.resistance = resistance;
            economy::city_totals_squatted(state, &variant, item, None, &self.key.squatted)
        };
        if let Some((_, totals)) = self.totals.iter().find(|(k, _)| *k == slot) {
            debug_assert_eq!(*totals, fresh(), "cached city totals for {:?}", city.id);
            return *totals;
        }
        let totals = fresh();
        self.totals.push((slot, totals));
        totals
    }

    /// `city_yields_mode(state, variant, None, None, squatted, full)` where the variant is
    /// `city` with `installed` built (`economy::install_building`), or `city` itself.
    pub(crate) fn yields(
        &mut self,
        state: &GameState,
        city: &City,
        installed: Option<BuildingKind>,
        full: bool,
    ) -> economy::CityYields {
        let slot = (installed, full);
        let fresh = || {
            let mut variant = city.clone();
            if let Some(building) = installed {
                economy::install_building(&mut variant, building);
            }
            economy::city_yields_mode(state, &variant, None, None, &self.key.squatted, full)
        };
        if let Some((_, yields)) = self.yields.iter().find(|(k, _)| *k == slot) {
            debug_assert_eq!(*yields, fresh(), "cached city yields for {:?}", city.id);
            return yields.clone();
        }
        let yields = fresh();
        self.yields.push((slot, yields.clone()));
        yields
    }

    /// `economy::city_yields_base_mode(state, city, squatted, full)`: the yields before the
    /// build-dependent part (`economy::city_yields_for_item` adds it).
    pub(crate) fn base(
        &mut self,
        state: &GameState,
        city: &City,
        full: bool,
    ) -> economy::CityYields {
        let fresh = || economy::city_yields_base_mode(state, city, &self.key.squatted, full);
        if let Some((_, yields)) = self.base.iter().find(|(k, _)| *k == full) {
            debug_assert_eq!(
                *yields,
                fresh(),
                "cached city base yields for {:?}",
                city.id
            );
            return yields.clone();
        }
        let yields = fresh();
        self.base.push((full, yields.clone()));
        yields
    }
}

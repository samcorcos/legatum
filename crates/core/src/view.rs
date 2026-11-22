//! What one player can see: fog applied, teammates share vision. The UI and the AI both read
//! only this, so the AI can never cheat.

use crate::combat;
use crate::data::{self, CITY_HP, FOOD_PER_POP};
use crate::economy::{self, CityYields};
use crate::forecast::{self, BankruptcyForecast, ProductionStall};
use crate::hex::Hex;
use crate::objectives;
use crate::scoring;
use crate::state::{AiDifficulty, City, CityMemory, GameState, Unit};
use crate::types::*;
use crate::visibility;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[cfg(test)]
mod forecast_tests;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileView {
    pub explored: bool,
    /// Currently in sight of this player's team.
    pub visible: bool,
    /// Known once explored.
    pub terrain: Option<Terrain>,
    /// An unexplored ruin, if known.
    pub ruin: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wonder: Option<crate::wonders::Wonder>,
    /// Built landmark, public on explored land even while the tile is in fog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub landmark: Option<crate::landmarks::Landmark>,
    /// The wonder's one-time discovery reward has been paid, to someone. Public, like the wonder.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub wonder_found: bool,
    /// Current owner on explored land.
    pub owner: Option<PlayerId>,
    /// A map resource the viewer knows about (G11): bonus resources on every explored tile,
    /// strategic ones once the viewer knows the revealing technology. Deposits never move, so
    /// fogged tiles show them like terrain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<crate::resources::Resource>,
    /// An improvement on the tile's resource, last seen (`Tile::improved`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub improvement: Option<TileImprovement>,
    /// The tile's road links (`Tile::road`): public, so known on every explored tile and always
    /// current (roads are laid for everyone to see, and never removed). 0 = none.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub road: u8,
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

fn is_zero_u8(v: &u8) -> bool {
    *v == 0
}

impl TileView {
    /// Movement cost to enter this known tile (`Map::move_cost`: a road tile costs 1); `None`
    /// when its terrain is unknown or impassable.
    pub fn move_cost(&self) -> Option<i32> {
        self.terrain
            .and_then(Terrain::move_cost)
            .map(|c| if self.road != 0 { 1 } else { c })
    }
}

/// A built tile improvement as the viewer sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileImprovement {
    pub kind: ImprovementKind,
    /// The upgraded form (Deep Mine...): known only for the viewer's own tiles.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub upgraded: bool,
}

/// What building an improvement does, for its production option.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovementPreview {
    pub tile: Hex,
    pub resource: crate::resources::Resource,
    /// What the tile gains when built, at the owner's current technology.
    pub gain: Yields,
    /// The automatic upgrade the owner doesn't have yet: its name, technology and extra yields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upgrade: Option<ImprovementUpgrade>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonderPreview {
    pub tile: Hex,
    /// Income this tile stops giving when the wonder completes.
    pub lost: Yields,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovementUpgrade {
    pub tech: TechId,
    pub name: String,
    pub gain: Yields,
}

/// Cloning a view shares its fogged map until a test or fixture edits a tile.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SharedTileViews(
    Arc<Vec<TileView>>,
    /// The known-terrain map (`terrain_map`), built on first use and shared by clones. Any
    /// mutable access to the tiles drops it.
    #[serde(skip)]
    Arc<std::sync::OnceLock<crate::state::Map>>,
);

impl PartialEq for SharedTileViews {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SharedTileViews {}

impl SharedTileViews {
    /// The map as the viewer knows it: known terrain, Grassland where unexplored, no owners.
    pub fn terrain_map(&self, width: i32, height: i32) -> &crate::state::Map {
        self.1.get_or_init(|| {
            let mut map = crate::state::Map::new(width, height, Terrain::Grassland);
            for (tile, known) in map.tiles.iter_mut().zip(self.0.iter()) {
                tile.terrain = known.terrain.unwrap_or(Terrain::Grassland);
                tile.road = known.road;
                map.roaded |= known.road != 0;
            }
            map
        })
    }

    fn edit(&mut self) -> &mut Vec<TileView> {
        self.1 = Default::default();
        Arc::make_mut(&mut self.0)
    }
}

impl From<Vec<TileView>> for SharedTileViews {
    fn from(tiles: Vec<TileView>) -> Self {
        Self(Arc::new(tiles), Default::default())
    }
}

impl Deref for SharedTileViews {
    type Target = Vec<TileView>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SharedTileViews {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.edit()
    }
}

impl<'a> IntoIterator for &'a SharedTileViews {
    type Item = &'a TileView;
    type IntoIter = std::slice::Iter<'a, TileView>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut SharedTileViews {
    type Item = &'a mut TileView;
    type IntoIter = std::slice::IterMut<'a, TileView>;

    fn into_iter(self) -> Self::IntoIter {
        self.edit().iter_mut()
    }
}

/// A view part that AI projections clone often but rarely edit: cloning shares it, and the
/// first edit copies it (like `SharedTileViews`). Serialized exactly as the inner value.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Shared<T>(Arc<T>);

impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Clone> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T {
        Arc::make_mut(&mut self.0)
    }
}

impl<T> FromIterator<T> for Shared<Vec<T>> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(Arc::new(iter.into_iter().collect()))
    }
}

impl<T: Clone> IntoIterator for Shared<Vec<T>> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        Arc::unwrap_or_clone(self.0).into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Shared<Vec<T>> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T: Clone> IntoIterator for &'a mut Shared<Vec<T>> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Arc::make_mut(&mut self.0).iter_mut()
    }
}

#[cfg(test)]
#[test]
fn shared_tiles_clone_copies_only_on_write_and_keeps_json_array() {
    let tile = TileView {
        explored: true,
        visible: true,
        terrain: Some(Terrain::Grassland),
        ruin: false,
        wonder: None,
        landmark: None,
        wonder_found: false,
        owner: None,
        resource: None,
        improvement: None,
        road: 0,
    };
    let original: SharedTileViews = vec![tile.clone()].into();
    let mut projected = original.clone();
    assert!(Arc::ptr_eq(&original.0, &projected.0));
    assert_eq!(
        serde_json::to_string(&original).unwrap(),
        serde_json::to_string(&vec![tile]).unwrap()
    );
    projected[0].visible = false;
    assert!(!Arc::ptr_eq(&original.0, &projected.0));
    assert!(original[0].visible);
    assert_eq!(
        serde_json::from_str::<SharedTileViews>(&serde_json::to_string(&projected).unwrap())
            .unwrap(),
        projected
    );
}

/// The Siege upgrade of one of the viewer's units (`rules::siege`): the Gold price and, when it
/// cannot be bought now, the reason in words (technology, Mongol era, territory, Gold, Iron).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiegeOffer {
    pub price: Breakdown,
    pub blocker: Option<String>,
    /// Gold is the only thing missing: it can be bought as soon as the treasury has the price.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub waits_for_gold: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitView {
    pub id: UnitId,
    pub owner: PlayerId,
    pub kind: UnitKind,
    pub pos: Hex,
    pub hp: i32,
    pub moves_left: i32,
    pub max_moves: i32,
    /// Only for this player's own units.
    pub goto: Option<Hex>,
    /// Only the viewer's own confirmed route; other players' plans are private.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<Hex>,
    /// Private intent, visible only to the owner. Absent in older saves/views.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automation: Option<Automation>,
    /// Private standing order; the combat defense bonus below is independent of it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standing_order: Option<StandingOrder>,
    /// Owner-only request for a new decision, even if this unit is fortified.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub needs_orders: bool,
    /// Own units only: the enemy unit an attack order pursues (see `Unit::attack_target`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_target: Option<UnitId>,
    pub fortify: i32,
    /// Own units only; creation times of foreign units are private.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_turn: Option<u32>,
    pub strength: i32,
    pub ranged: i32,
    pub range: i32,
    pub military: bool,
    /// Promotion level (0..=5, or 6 for Sparta), public like Civ 6's promotion pips: every forecast against
    /// this unit includes it.
    #[serde(default)]
    pub level: u8,
    /// Own military units only: cumulative combat experience.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xp: Option<u32>,
    /// Own military units only: cumulative XP for the next level; None at the maximum level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xp_next: Option<u32>,
    /// Own units only: the civ this unit is locked in its city against, until Peace with it
    /// (`rules::occupation::lock`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occupation_lock: Option<PlayerId>,
    /// Gold to upgrade now, if possible (own units in own territory with the tech).
    pub upgrade_cost: Option<i32>,
    /// Newest researched upgrade and the paid forms along the way.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upgrade_to: Option<UnitKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upgrade_steps: Vec<UnitKind>,
    #[serde(default)]
    pub upgrade_breakdown: Option<Breakdown>,
    /// Own units: the strategic supply the upgrade also uses (Phalangite→Pikeman: 1 Iron).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upgrade_need: Option<crate::resources::Resource>,
    /// Any visible unit short of the strategic supply it needs: it fights with
    /// `resources::UNSUPPLIED_STRENGTH_PERCENT` less strength until supply returns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupplied: Option<crate::resources::Resource>,
    /// The strength an unsupplied unit loses in every fight (the forecast's "No Iron supply"
    /// line): show `strength - unsupplied_loss`, "27, was 36: no Iron".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupplied_loss: Option<i32>,
    /// Own units: why the upgrade can't take the supply it needs now (the build card's sentence).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upgrade_supply_blocker: Option<String>,
    /// The unit has bought Siege (`Unit::siege`): its attacks on a city ignore the Walls its owner's
    /// technology reaches. Public: the siege gear is drawn beside it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub siege: bool,
    /// Own land military units without Siege, once Mathematics is known: its price and why it
    /// cannot be bought now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub siege_offer: Option<SiegeOffer>,
    pub sight: Option<i32>,
    pub resting_heal: Option<Breakdown>,
    /// An Heir only (public): the Byzantium empire whose goal he carries, whoever holds him now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heir_of: Option<PlayerId>,
    /// A hired Mercenary Company (public): the last turn of its contract, and the faction that
    /// sold it. The countdown is on the unit for everyone (`company`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_until: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Faction>,
}

/// Military upkeep effect of building one more of this unit, for the production picker card
/// (legatum-ui: "every build card shows its ongoing cost"). Absent for civilian units,
/// buildings and projects, which have no upkeep.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpkeepPreview {
    /// True while the empire still has a free military slot for this unit.
    pub free: bool,
    /// Free military slots left before this unit (0 once all are used).
    pub free_slots_left: i32,
    /// Gold/turn this unit costs once paid (1, or 3 for the Switzerland).
    pub rate: i32,
    /// The empire's net Gold/turn after building this unit.
    pub gold_per_turn_after: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionOption {
    pub item: ProductionItem,
    pub cost: i32,
    /// Progress this item would have if selected now: its own progress when active, else its
    /// banked progress plus any unassigned Production (overflow, idle banking).
    pub production_stored: i32,
    /// Progress banked for this item from an earlier switch (0 for the active build).
    #[serde(default)]
    pub production_banked: i32,
    /// Full purchase price, independent of the active build and stored progress.
    pub buy_cost: Option<i32>,
    #[serde(default)]
    pub buy_blocker: Option<String>,
    /// Why buying is blocked, as data (`buy_blocker` words it). The AI reads this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_block: Option<economy::BuyBlock>,
    #[serde(default)]
    pub buy_quote: Option<economy::BuyQuote>,
    /// Income with this item's automatic tile priorities, at the current population.
    pub yields: CityYields,
    /// Turns at this item's production rate, including transferable stored progress and purchases of
    /// this item. Assumes sufficient population and space to spawn a completed unit.
    /// -1 during resistance or when more production is needed and none accumulates.
    pub turns: i32,
    /// Strategic supply this item uses for as long as it exists (1 of this resource).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supply_need: Option<crate::resources::Resource>,
    /// Why this item can't start now for lack of that supply, with how to get more. It can
    /// still be queued behind other work; the queue starts it once supply is spare.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_blocker: Option<String>,
    /// Military upkeep effect of building this unit now; None for civilians, buildings and
    /// projects (no upkeep).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upkeep: Option<UpkeepPreview>,
    /// For an Improvement item: its tile and what it adds (plus the upgrade still to come).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub improvement: Option<ImprovementPreview>,
    /// For a tile wonder: the yield replaced on completion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wonder: Option<WonderPreview>,
}

/// A researched item the city can't choose now, and why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnavailableOption {
    pub item: ProductionItem,
    pub cost: i32,
    pub reason: String,
    /// Missing in-city predecessor for a building upgrade. Kept as data so the UI does not
    /// infer upgrade paths from the wording of `reason`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prerequisite_building: Option<BuildingKind>,
    /// Progress banked for this item, waiting until the city can build it again.
    pub production_banked: i32,
}

/// A building tier yielding nothing for lack of strategic supply ("Factory idle: no Coal").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdleBuilding {
    pub building: BuildingKind,
    pub resource: crate::resources::Resource,
}

/// Full detail, only for this player's own cities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityDetail {
    /// The owner took this city from another founder, so a Raze control is relevant.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub captured: bool,
    /// Original capital status survives capture; replacement capitals do not have it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub original_capital: bool,
    /// Last owner, owner-only while the city is held; used to value denial.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub former_owner: Option<PlayerId>,
    /// Why a captured city cannot be razed now; None means Raze is legal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raze_blocker: Option<String>,
    pub yields: CityYields,
    #[serde(default)]
    pub idle_yields: Option<CityYields>,
    pub food_stored: i32,
    pub growth_threshold: i32,
    #[serde(default)]
    pub growth_breakdown: Breakdown,
    /// None when the city is not growing.
    pub turns_to_grow: Option<i32>,
    pub producing: Option<ProductionItem>,
    #[serde(default)]
    pub production_queue: Vec<ProductionItem>,
    /// A rival completed the world wonder this city was building; it waits for a new choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wonder_lost: Option<crate::state::WonderLoss>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_automation: Option<crate::types::CityBuildFocus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_auto_pick: Option<ProductionItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_queued: Option<ProductionItem>,
    pub production_stored: i32,
    pub production_cost: Option<i32>,
    pub turns_to_complete: Option<i32>,
    /// Egypt's building is held until its population requirement returns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub food_construction_paused: Option<String>,
    /// Why a unit with enough production cannot complete next turn, after growth.
    pub production_stall: Option<ProductionStall>,
    /// Most Production banked while nothing is being built: the idle cap with an
    /// empty plan, or the first queued item's cost while the queue waits.
    pub idle_production_cap: i32,
    /// Production lost next turn if this city stays idle or waiting, at current yields.
    pub idle_production_waste: i32,
    pub buy_cost: Option<i32>,
    pub buildings: Vec<BuildingKind>,
    /// Building tiers idle for lack of strategic supply: they yield nothing until it returns.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub idle_buildings: Vec<IdleBuilding>,
    pub options: Vec<ProductionOption>,
    /// Researched units and buildings this city can't choose or queue right now, with the
    /// reason (a Settler at population 1, a building's population gate or missing lower
    /// tier). Shown greyed out; Great Works have their own section and are not listed.
    #[serde(default)]
    pub unavailable: Vec<UnavailableOption>,
    /// Progress banked per item from earlier switches, sorted by item.
    #[serde(default)]
    pub production_banked: Vec<crate::state::BankedProduction>,
    /// Every tile the city owns, its centre included. Those within `data::CITY_WORK_RADIUS`
    /// are worked (`yields.worked`, plus the centre); the rest only block rivals.
    pub territory: Vec<Hex>,
    /// The tile the city claims when it next grows (`rules::cities::next_claim`), or None
    /// when no unowned tile touches its territory.
    pub next_claim: Option<ClaimPreview>,
    /// The price before the ring surcharge: `TILE_GOLD_BASE` plus `TILE_GOLD_PER_TILE` per tile
    /// the city gained since founding (grown or bought). A tile costs this times
    /// `100 + TILE_RING_STEP_PERCENT * (ring - 1)` percent.
    #[serde(default)]
    pub tile_base_price: i32,
    /// Explored unowned tiles touching this city's borders within `data::CITY_WORK_RADIUS`,
    /// nearest ring first, each with its price (`Order::BuyTile`). Tiles beyond the radius only
    /// come from growth. Empty in a decision view.
    #[serde(default)]
    pub buyable_tiles: Vec<BuyableTile>,
    /// Why this city cannot buy any tile this turn (its purchase is used, an occupation freeze,
    /// resistance, or no explored tile to buy), or None. Gold is not part of it: each tile's
    /// price is in `buyable_tiles`. None in a decision view.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_tile_blocker: Option<String>,
    pub resistance: i32,
    /// Turns until starvation costs a population at current yields: stored Food runs out and the
    /// next deficit shrinks the city. None while fed, resisting, or at population 1 (which cannot
    /// shrink).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turns_to_starve: Option<i32>,
    /// Turns until the city is back to full HP if nothing damages it, at `data::CITY_HEAL` per
    /// turn. None at full HP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turns_to_heal: Option<i32>,
    /// An at-war military unit stands next to the city, so it regains no HP at the end of this
    /// turn (it doesn't heal on a turn it takes damage either).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub heal_blocked: bool,
    /// The city took battle damage on the turn that just resolved (`City.last_attacked_turn`).
    /// Owner-only like the rest of the detail; the app's siege alerts read it on a loaded game,
    /// where no turn report exists.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub attacked_last_turn: bool,
    /// What this city can make only from a population (the Settler at 2, upgrade buildings), in
    /// population order. Lists every such item it could otherwise make, reached or not, so a
    /// growth notice can name what the new population unlocked.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub population_unlocks: Vec<PopulationUnlock>,
}

/// A tile a city can buy, with its ring and price.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuyableTile {
    pub tile: Hex,
    /// Hex distance from the city (1 to `data::CITY_WORK_RADIUS`).
    pub ring: i32,
    /// Gold to buy it now.
    pub price: i32,
}

/// The tile a city's next growth claims, with what it adds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimPreview {
    pub tile: Hex,
    /// The tile's yields for the owner (terrain plus known resources).
    pub yields: Yields,
    /// Within the work radius, so the city works it as soon as it is claimed; otherwise it only
    /// blocks rivals.
    pub worked: bool,
}

/// An item gated by city population.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopulationUnlock {
    pub population: i32,
    pub item: ProductionItem,
}

/// The population an item needs before it can start (Settler) or begin (upgrade buildings).
pub fn item_min_population(item: ProductionItem) -> i32 {
    match item {
        ProductionItem::Unit {
            unit: UnitKind::Settler,
        } => data::SETTLER_MIN_POP,
        ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
            data::building_min_population(building)
        }
        _ => 0,
    }
}

/// An aircraft of the viewer's team, in a city's hangar (`rules::air`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AircraftView {
    pub id: UnitId,
    pub owner: PlayerId,
    pub kind: UnitKind,
    pub city: CityId,
    pub hp: i32,
    /// Promotion level (0..=5, or 6 for Sparta); every strike and interception forecast includes it.
    #[serde(default)]
    pub level: u8,
    /// Cumulative combat experience.
    #[serde(default)]
    pub xp: u32,
    /// Cumulative XP for the next level; None at the maximum level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xp_next: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upgrade_cost: Option<i32>,
    /// Every line it can upgrade along now, the default first (`data::upgrade_choices`): one
    /// for most aircraft, a Fighter and a Bomber for a Biplane once Air Power is known. Empty
    /// when it cannot upgrade.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upgrade_options: Vec<AircraftUpgradeOption>,
    /// The supply missing at its base; an unsupplied plane strikes and intercepts at a penalty.
    #[serde(default)]
    pub unsupplied: Option<crate::resources::Resource>,
}

/// One line an aircraft can upgrade along now (`Order::UpgradeAircraft::to` is `via`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AircraftUpgradeOption {
    /// The first form on this line (Fighter or Bomber for a Biplane).
    pub via: UnitKind,
    /// The newest researched form it becomes.
    pub to: UnitKind,
    /// Gold for the whole upgrade.
    pub cost: i32,
}

/// Aircraft in a rival hangar currently visible to this player's team. A sighting exposes
/// only the kind and base, never aircraft health, supply or upgrade information.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AirSighting {
    pub owner: PlayerId,
    pub kind: UnitKind,
    pub city: CityId,
}

/// The aircraft in the hangars of `team`'s cities.
pub fn aircraft_views(state: &GameState, team: TeamId) -> Vec<AircraftView> {
    state
        .aircraft
        .iter()
        .filter_map(|a| {
            let owner = state.cities.get(&a.city)?.owner;
            (state.team_of(owner) == team).then_some(AircraftView {
                id: a.id,
                owner,
                kind: a.kind,
                city: a.city,
                hp: a.hp,
                level: crate::rules::promotion::level_for_rules(
                    state.setup.rule_experiments.faction_rules_off,
                    state.player(owner).faction,
                    a.xp,
                ),
                xp: a.xp,
                xp_next: crate::rules::promotion::next_level_xp_for_rules(
                    state.setup.rule_experiments.faction_rules_off,
                    state.player(owner).faction,
                    a.xp,
                ),
                upgrade_cost: crate::rules::air::upgrade_price(state, owner, a.id, None)
                    .ok()
                    .map(|(_, cost)| cost),
                upgrade_options: crate::data::upgrade_choices(a.kind)
                    .into_iter()
                    .filter_map(|via| {
                        crate::rules::air::upgrade_price(state, owner, a.id, Some(via))
                            .ok()
                            .map(|(to, cost)| AircraftUpgradeOption { via, to, cost })
                    })
                    .collect(),

                unsupplied: crate::resources::unit_need(a.kind).filter(|_| {
                    crate::resources::is_idle(
                        state.player(owner),
                        crate::resources::SupplyUser::Unit { unit: a.id },
                    )
                }),
            })
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityView {
    pub id: CityId,
    pub owner: PlayerId,
    pub name: String,
    pub pos: Hex,
    pub population: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub strength: i32,
    pub is_capital: bool,
    /// Original founding capital, even after its capital status changes hands.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub original_capital: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_capital_of: Option<PlayerId>,
    /// The empire that founded this city (public: founding is in the chronicle). A Celtic
    /// capture of a city its own founder retakes is not a sack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub founder: Option<PlayerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_bounty: Option<i32>,
    /// This viewer has not yet been paid for capturing this Athenian city.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub greece_captive_tech: bool,
    /// An own Athenian city's remaining free-technology liability to at least one rival.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub greece_captive_at_risk: bool,
    pub walls: bool,
    #[serde(default)]
    pub citadel: bool,
    #[serde(default)]
    pub star_fort: bool,
    /// Whether the last visible observation showed a Workshop or its upgrade.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub has_workshop: bool,
    /// Whether the last visible observation showed an Air Defence: it intercepts air strikes on
    /// this city (`rules::air`), and an air-strike forecast says so.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub has_air_defence: bool,
    /// Occupation lock: the civ a military unit of the viewer would be locked against
    /// by ending a move in this city (`rules::occupation::entering_lock`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occupation_lock: Option<PlayerId>,
    /// Completed Great Works are public for every explored city, even under fog.
    #[serde(default)]
    pub great_works: Vec<BuildingKind>,
    /// A chosen wonder tile under construction. Public at the city, including through fog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub construction: Option<ConstructionView>,
    /// The city is building Walls. Public at the city, including through fog, like a wonder site.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub walls_building: bool,
    /// Currently in sight (otherwise only identity and ownership are current).
    pub visible: bool,
    pub detail: Option<Shared<CityDetail>>,
}

impl CityView {
    /// The Walls tier above plain Walls: 0 none, 1 Castle, 2 Star Fort.
    pub fn fort_tier(&self) -> u8 {
        u8::from(self.citadel) + u8::from(self.star_fort)
    }

    fn observed(
        city: &CityMemory,
        pos: Hex,
        visible: bool,
        detail: Option<CityDetail>,
        faction: Faction,
        rules: crate::experiments::RuleExperiments,
        era: u8,
    ) -> Self {
        Self {
            id: city.id,
            owner: city.owner,
            name: city.name.clone(),
            pos,
            population: city.population,
            hp: city.hp,
            max_hp: CITY_HP,
            strength: combat::city_defense_for_rules(
                rules,
                faction,
                city.population,
                city.walls,
                city.fort_tier(),
                city.hp,
                era,
            )
            .total,
            is_capital: city.is_capital,
            original_capital: false,
            original_capital_of: None,
            founder: None,
            capture_bounty: None,
            greece_captive_tech: false,
            greece_captive_at_risk: false,
            walls: city.walls,
            citadel: city.citadel,
            star_fort: city.star_fort,
            has_workshop: city.has_workshop,
            has_air_defence: city.has_air_defence,
            occupation_lock: None,
            great_works: Vec::new(),
            construction: None,
            walls_building: false,
            visible,
            detail: detail.map(Shared::from),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructionView {
    pub building: BuildingKind,
    pub tile: Hex,
    /// 0 plot, 1 footing, 2 half built, 3 roofline.
    pub stage: u8,
    /// Thousandths of the current item's production cost, for the step between turns.
    pub progress: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerPublic {
    pub id: PlayerId,
    pub name: String,
    pub faction: Faction,
    pub team: TeamId,
    pub is_human: bool,
    /// The AI level selected at game creation. Normal is omitted for old-view compatibility.
    #[serde(default, skip_serializing_if = "AiDifficulty::is_normal")]
    pub difficulty: AiDifficulty,
    /// Hard + Search uses rollouts; only meaningful for a Hard AI.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub search: bool,
    pub alive: bool,
    pub vp: u32,
    /// Every VP this empire scores, by typed source (`VpSource`).
    pub vp_sources: Vec<(VpSource, u32)>,
    /// Public: the rival original capitals this empire holds now, each worth 1 VP while held
    /// (`scoring::capitals_held`). A capital retaken or razed leaves the list.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capitals_taken: Vec<(CityId, String)>,
    /// Public: the rival original capitals this Celtic empire burned in a sack, each worth 1 VP
    /// for the rest of the game (`Player::capitals_sacked`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capitals_sacked: Vec<(CityId, String)>,
    /// Whether this empire's original capital still stands in its own team's hands.
    /// Public like the treasuries: it is the capitals victory's whole race
    /// (`scoring::capital_stands`). Absent in older views, read as standing.
    #[serde(default = "capital_stands_default")]
    pub capital_stands: bool,
    pub cities: u32,
    pub faction_goal_done: bool,
    /// Sakoku: the country is closed, with the turn ends it has been (public).
    #[serde(default)]
    pub sakoku_turns: Option<u32>,
    /// This empire is building its royal tomb (public): none of its cities builds or grows.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tomb_work: bool,
    /// Turn ends of work this empire has put toward its royal tomb (public).
    #[serde(default, skip_serializing_if = "crate::state::is_zero_u32")]
    pub tomb_turns: u32,
    /// The rule this empire's goal follows (public; absent in views from before it existed:
    /// use [`PlayerPublic::rule`]).
    #[serde(default)]
    pub goal_rule: Option<crate::goals::GoalRule>,
    /// The empire's current era, **1-based** (1 Ancient .. 8 Future; `ResearchView::era` is the
    /// same era 0-based). Public. It rises the turn the previous era's last technology lands.
    #[serde(default)]
    pub era: u8,
    /// The empire's current Gold, public to every player (rules 31: every treasury is visible; 0
    /// once dead).
    #[serde(default)]
    pub gold: i32,
    /// Public economy value: Gold plus Production across the empire's cities (0 once dead).
    #[serde(default)]
    pub economy: i32,
    /// Public army strength: combat value of its units and aircraft (0 once dead).
    #[serde(default)]
    pub army: i32,
    /// Public technological progress: furthest era entered (times 1000) plus technologies known
    /// (0 once dead).
    #[serde(default)]
    pub advancement: i32,
    #[serde(default)]
    pub power: i32,
    #[serde(default)]
    pub lead_percent: i32,
    /// Decadence, the percent cut from every yield of the empire's cities.
    #[serde(default)]
    pub decadence_percent: i32,
    /// Thin levies (Byzantium only): armed units fielded and the cap. Public.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub levy: Option<LevyView>,
    /// Hired Mercenary Companies this empire fields (public, like the countdowns on the units).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub companies: u32,
}

fn capital_stands_default() -> bool {
    true
}

/// An empire's armed units against its thin-levy cap (`data::levy_cap`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevyView {
    pub units: u32,
    pub cap: u32,
}

impl PlayerPublic {
    /// The rule this empire's goal follows.
    pub fn rule(&self) -> crate::goals::GoalRule {
        self.goal_rule
            .unwrap_or_else(|| crate::goals::default_rule(self.faction))
    }
}

/// One public World Age card.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveView {
    /// The card's slot in its Age: always 0 (one card per Age).
    pub slot: u8,
    pub family: crate::objectives::Family,
    pub text: String,
    pub target: u32,
    /// The rule behind the text (the AI keys on it).
    pub objective: crate::objectives::Objective,
}

/// Who scored an era, and with which objective slot (public).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraScoredView {
    pub player: PlayerId,
    pub slot: u8,
    pub turn: u32,
}

/// An era's public objectives and who has entered and scored it (VP v2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraView {
    pub era: u8,
    pub name: String,
    pub objectives: Vec<ObjectiveView>,
    pub entered_by: Vec<PlayerId>,
    pub scored_by: Vec<EraScoredView>,
}

/// The viewer's own progress on one World Age card.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveProgress {
    pub era: u8,
    pub slot: u8,
    pub current: u32,
    pub target: u32,
    /// The viewer can score the era: it has entered it, or in a World Age game the Age has begun.
    pub entered: bool,
    /// The viewer has already scored this era.
    pub era_scored: bool,
    /// The turn the Age began (when counting started).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entered_turn: Option<u32>,
    /// The card is met and scores at turn end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready: Option<bool>,
}

/// The viewer's own accounting for a gain-count objective. Kept separate from the progress
/// contract used by AI planning and its fixture builders.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveAccounting {
    pub era: u8,
    pub slot: u8,
    pub baseline: u32,
    pub total: u32,
    pub counting_since_turn: u32,
    /// Other entered, unscored eras whose objectives restart if this era scores.
    pub restart_eras: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EconomyView {
    pub gold: i32,
    #[serde(default)]
    pub military_purchases: u32,
    pub gold_per_turn: Breakdown,
    pub science_per_turn: Breakdown,
    pub upkeep: i32,
    #[serde(default)]
    pub upkeep_breakdown: Breakdown,
    /// Current queues and positions, including upkeep of completing units. Orders and
    /// simultaneous movement/combat can change the outcome.
    pub bankruptcy: Option<BankruptcyForecast>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechOption {
    pub tech: TechId,
    pub cost: Breakdown,
    pub turns: Option<i32>,
    /// The era (0-based) the technology belongs to.
    pub era: u8,
    /// Whether the empire may research it now (unknown and in its current era).
    pub researchable: bool,
    pub income_change: Vec<economy::YieldSource>,
    #[serde(default)]
    pub effects: crate::research::TechEffects,
    /// Other living empires that know it (its diffusion discount). The AI reads this.
    #[serde(default)]
    pub known_by: i32,
}

/// One queued technology's arrival forecast (`ResearchView::schedule`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleSlot {
    pub tech: TechId,
    /// Its Science cost now (diffusion included).
    pub cost: i32,
    /// Turns from now until it completes, after the queued ones before it and the stored
    /// Science; None with no Science income.
    pub turns: Option<i32>,
    pub arrival_turn: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchView {
    #[serde(default)]
    pub science_conversion: ScienceConversion,
    /// The empire's current era (the strict gate: only this era's technologies can be researched).
    #[serde(default)]
    pub era: u8,
    pub era_name: String,
    /// Technologies in the current era, and how many of them are known.
    #[serde(default)]
    pub era_total: u32,
    #[serde(default)]
    pub era_known: u32,
    /// The first empire into each era, public (`GameState::era_firsts`), as (era, empire).
    #[serde(default)]
    pub era_firsts: Vec<(u8, PlayerId)>,
    /// Arrival forecast of the queued order: when each queued technology completes at the
    /// current Science income, in queue order.
    #[serde(default)]
    pub schedule: Vec<ScheduleSlot>,
    pub current: Option<TechId>,
    /// Includes the current technology, also for saves predating queues.
    #[serde(default)]
    pub queue: Vec<TechId>,
    pub progress: i32,
    pub cost: Option<i32>,
    pub turns_left: Option<i32>,
    pub known: Vec<TechId>,
    /// Technologies this faction skips: known from the start because they give it nothing
    /// (`data::skipped_techs`). Research screens leave them out, and the era counts above
    /// exclude them.
    #[serde(default)]
    pub skipped: Vec<TechId>,
    pub available: Vec<TechOption>,
    /// Every existing technology, including locked nodes, at current diffusion prices.
    pub catalog: Vec<TechOption>,
    pub unlocks: Vec<crate::research::TechUnlock>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamStanding {
    pub team: TeamId,
    pub members: Vec<PlayerId>,
    pub vp: u32,
    pub target: u32,
    /// Set while the team holds its VP target: it wins when `turns_held` reaches `turns_needed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub victory_hold: Option<VictoryHold>,
}

/// A team's progress towards winning by holding its VP target (public to every player).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VictoryHold {
    /// Consecutive turn ends at or above the target, counting the turn it was reached (1 then).
    pub turns_held: u32,
    pub turns_needed: u32,
}

/// A standing Yam Relay Station: Mongols empire `of`'s, in `city`, held by `holder` (its owner).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierHallView {
    pub of: PlayerId,
    pub holder: PlayerId,
    pub city: CityId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile: Option<Hex>,
}

/// The Imperial Diwan's original India empire, current city and holder; public like the Yam Relay Station.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImperialDiwanView {
    pub of: PlayerId,
    pub holder: PlayerId,
    pub city: CityId,
}

/// A city that carries a faction goal's 2 VP for whoever holds it (Rome's Colonia Victrix,
/// Egypt's royal tomb): the empire whose goal it is, the holder, and the city with its place and
/// name. Public to every empire, in fog and on unexplored land too, like the Heir.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalCityView {
    pub of: PlayerId,
    pub holder: PlayerId,
    pub city: CityId,
    pub at: Hex,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerView {
    #[serde(
        default,
        skip_serializing_if = "crate::experiments::RuleExperiments::is_default"
    )]
    pub rule_experiments: crate::experiments::RuleExperiments,
    /// The game started with the AI-only teams allied in pairs (`GameSetup::ai_alliances`).
    /// Public setup fact; omitted when false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ai_alliances: bool,
    pub diplomacy: Shared<crate::diplomacy::DiplomacyView>,
    pub turn: u32,
    pub turn_limit: Option<u32>,
    pub me: PlayerId,
    pub team: TeamId,
    pub width: i32,
    pub height: i32,
    /// Row-major, same indexing as the map.
    pub tiles: SharedTileViews,
    /// Own and teammates' units, visible enemies, plus ordinary attackers revealed through this
    /// team's next planning turn.
    pub units: Shared<Vec<UnitView>>,
    /// Own and teammates' cities, plus enemy cities on explored tiles.
    pub cities: Shared<Vec<CityView>>,
    /// Only this player's failed founding locations; no enemy city or rejection reason.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub failed_city_sites: BTreeSet<Hex>,
    /// Explored, otherwise-foundable tiles inside the real founding gap of any city. Only the blocked tile is exposed;
    /// a city on unexplored land contributes no identity, owner or center position to the view.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub city_gap_blocked: BTreeSet<Hex>,
    /// Recent sightings, not live units or valid combat targets. Each has an observation turn.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_threats: Vec<crate::state::ThreatObservation>,
    /// Barbarian camps this team sees or remembers, in hex order. Empty with barbarians off.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub barbarian_camps: Vec<BarbarianCampView>,
    pub players: Shared<Vec<PlayerPublic>>,
    pub economy: Shared<EconomyView>,
    /// Strategic supply, one per strategic resource, revealed or not.
    #[serde(default)]
    pub supply: Vec<crate::resources::SupplyView>,
    pub research: Shared<ResearchView>,
    /// Own ground army's exact contributions; omitted from the AI's lightweight view.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub army_breakdown: Vec<crate::army::Contribution>,
    /// VP v2: every era's public objectives and who has scored them. Empty with VP v2 off.
    #[serde(default)]
    pub eras: Shared<Vec<EraView>>,
    /// VP v2: the viewer's own progress on the objectives of the eras it has entered and the
    /// next one.
    #[serde(default)]
    pub objective_progress: Vec<ObjectiveProgress>,
    /// Own gain-count baselines and prospective effects on other open eras.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objective_accounting: Vec<ObjectiveAccounting>,
    #[serde(default)]
    pub development: Option<crate::projects::DevelopmentForecast>,
    /// Your own faction goal's progress (every faction). A Permanent goal has none once it is done;
    /// a Live or Held goal always has one. Rivals' progress is private.
    #[serde(default)]
    pub faction_goal: Option<crate::goals::FactionGoalView>,
    /// The viewer's one-time actions and pending free technologies (`rules::actions`).
    #[serde(default)]
    pub actions: crate::rules::actions::ActionsView,
    /// Every standing Yam Relay Station (public): who holds it and in which city. Its holder's land is
    /// open to every empire at Peace (`passage_allowed`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub courier_halls: Vec<CourierHallView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imperial_diwans: Vec<ImperialDiwanView>,
    /// Each Colonia Victrix on the map and who holds it. Public: rivals see the city is worth 2 VP.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub colonias: Vec<GoalCityView>,
    /// Each royal tomb on the map and who holds it. Public: rivals see the city is worth 2 VP.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub royal_tombs: Vec<GoalCityView>,
    /// Where each royal tomb under construction will stand (`goals::tomb_site`), for every
    /// empire whose work is on. Public, like the work and its count: rivals see the city to watch.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tomb_sites: Vec<GoalCityView>,
    /// Your team's aircraft, each in one city's hangar (`rules::air`); nobody else's are visible.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aircraft: Vec<AircraftView>,
    /// Rival aircraft based in cities currently in sight. Fogged hangars remain private.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub air_sightings: Vec<AirSighting>,
    /// Rival aircraft this team saw in hangars or battle reports, with last observation turn.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub air_contacts: Vec<crate::state::AirContact>,
    /// Byzantium only: every Mercenary Company it could hire now, by seller and kind (the shop;
    /// `company::offers`). Private; empty for other factions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub company_offers: Vec<crate::company::Offer>,
    #[serde(default)]
    pub great_works: Vec<crate::projects::GreatWorkView>,
    pub standings: Shared<Vec<TeamStanding>>,
    pub winner: Option<TeamId>,
    pub finished: bool,
    /// The game's per-turn record for the post-game report. Rivals' numbers stay hidden during
    /// play, so it is set only once the game is finished (and only in the `Full` view).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<crate::history::GameHistory>,
}

/// A barbarian camp as the viewer knows it: seen now, or remembered where last seen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarbarianCampView {
    pub pos: Hex,
    /// Currently in sight (otherwise last known).
    pub visible: bool,
}

impl PlayerView {
    /// A player's public army strength (0 for an unknown or dead empire).
    pub fn army_of(&self, p: PlayerId) -> i32 {
        self.players
            .iter()
            .find(|q| q.id == p)
            .map_or(0, |q| q.army)
    }

    /// A player's public technological progress (0 for an unknown or dead empire).
    pub fn advancement_of(&self, p: PlayerId) -> i32 {
        self.players
            .iter()
            .find(|q| q.id == p)
            .map_or(0, |q| q.advancement)
    }

    /// A player's public economy value (0 for an unknown or dead empire).
    pub fn economy_of(&self, p: PlayerId) -> i32 {
        self.players
            .iter()
            .find(|q| q.id == p)
            .map_or(0, |q| q.economy)
    }

    pub fn team_of(&self, player: PlayerId) -> TeamId {
        if player.is_barbarian() {
            return TeamId::BARBARIAN;
        }
        self.player_public(player).expect("known player").team
    }

    /// The public record of `player` (ids are unique). Players are listed by id, so the record
    /// is usually at index `player`; hand-built views may differ, so fall back to a search.
    fn player_public(&self, player: PlayerId) -> Option<&PlayerPublic> {
        match self.players.get(player.0 as usize) {
            Some(p) if p.id == player => Some(p),
            _ => self.players.iter().find(|p| p.id == player),
        }
    }

    pub fn same_team(&self, a: PlayerId, b: PlayerId) -> bool {
        self.team_of(a) == self.team_of(b)
    }

    pub fn at_war(&self, a: PlayerId, b: PlayerId) -> bool {
        self.teams_at_war(self.team_of(a), self.team_of(b))
    }

    /// The team that would waive an Alliance's upkeep between `a` and `b` this turn, from public
    /// information: a team either side is at War with that is alive and on a victory hold or
    /// within `scoring::NEAR_VICTORY_VP` of its target. Mirrors `diplomacy::upkeep_waived_by`.
    pub fn alliance_upkeep_waived_by(&self, a: TeamId, b: TeamId) -> Option<TeamId> {
        self.standings
            .iter()
            .map(|s| (s.team, s))
            .find_map(|(t, s)| {
                (t != a
                    && t != b
                    && (self.teams_at_war(a, t) || self.teams_at_war(b, t))
                    && self
                        .players
                        .iter()
                        .any(|p| p.alive && s.members.contains(&p.id))
                    && (s.victory_hold.is_some()
                        || s.vp.saturating_add(crate::scoring::NEAR_VICTORY_VP) >= s.target))
                    .then_some(t)
            })
    }

    pub fn teams_at_war(&self, a: TeamId, b: TeamId) -> bool {
        if a == TeamId::BARBARIAN || b == TeamId::BARBARIAN {
            return a != b;
        }
        let key = crate::diplomacy::pair(a, b);
        a != b
            && (self.diplomacy.legacy
                || self
                    .diplomacy
                    .relations
                    .iter()
                    .any(|r| (r.a, r.b) == key && r.state == crate::diplomacy::RelationState::War))
    }

    /// Mirrors `GameState::passage_allowed`: own team, War, an Alliance, land the mover's
    /// land of a
    /// Yam Relay Station's holder (open to every empire).
    pub fn passage_allowed(
        &self,
        player: PlayerId,
        _kind: UnitKind,
        owner: Option<PlayerId>,
    ) -> bool {
        owner.is_none_or(|o| {
            let (a, b) = (self.team_of(player), self.team_of(o));
            a == b
                || self.teams_at_war(a, b)
                || (!player.is_barbarian()
                    && !self.is_closed(o)
                    && (self.teams_allied(a, b)
                        || self.courier_halls.iter().any(|h| h.holder == o)))
        })
    }

    /// `o` has closed the country (Sakoku): public on every view.
    pub fn is_closed(&self, o: PlayerId) -> bool {
        self.players
            .iter()
            .any(|p| p.id == o && p.sakoku_turns.is_some())
    }

    /// `passage_allowed` for one mover and kind, precomputed for every known owner (see
    /// `EntryTable`).
    pub fn entry_table(&self, player: PlayerId, kind: UnitKind) -> EntryTable {
        let mut open = [None; 256];
        for p in &self.players {
            open[p.id.0 as usize] = Some(self.passage_allowed(player, kind, Some(p.id)));
        }
        open[PlayerId::BARBARIAN.0 as usize] =
            Some(self.passage_allowed(player, kind, Some(PlayerId::BARBARIAN)));
        EntryTable { player, kind, open }
    }

    pub fn allied(&self, a: PlayerId, b: PlayerId) -> bool {
        // Teams are only looked up when there is a relation to compare, as before.
        !self.diplomacy.relations.is_empty() && self.teams_allied(self.team_of(a), self.team_of(b))
    }

    fn teams_allied(&self, a: TeamId, b: TeamId) -> bool {
        self.diplomacy
            .relations
            .iter()
            .any(|r| r.between(a, b) && r.state == crate::diplomacy::RelationState::Alliance)
    }

    pub fn shares_vision(&self, a: PlayerId, b: PlayerId) -> bool {
        let (a, b) = (self.team_of(a), self.team_of(b));
        a == b
            || self
                .diplomacy
                .relations
                .iter()
                .any(|r| r.between(a, b) && crate::diplomacy::shares_sight(r))
    }

    /// `path::exit_distances` over this view's map with every known city a wall (a unit stranded
    /// in closed land never routes out through a foreign city: it could not stop there).
    pub fn exit_distances(&self, map: &crate::state::Map, open: impl Fn(Hex) -> bool) -> Vec<i32> {
        let cities: std::collections::BTreeSet<Hex> = self.cities.iter().map(|c| c.pos).collect();
        crate::path::exit_distances_around(map, open, |h| cities.contains(&h))
    }

    /// Only known owners constrain this route; unknown terrain is potentially passable.
    pub fn movement_path(
        &self,
        owner: PlayerId,
        kind: UnitKind,
        from: Hex,
        to: Hex,
    ) -> Option<Vec<Hex>> {
        self.movement_path_with(&self.entry_table(owner, kind), from, to)
    }

    /// `movement_path` for the mover of `entry` (`entry_table(owner, kind)`), for callers
    /// routing many units of one owner and kind on one view.
    pub fn movement_path_with(&self, entry: &EntryTable, from: Hex, to: Hex) -> Option<Vec<Hex>> {
        let map = self.tiles.terrain_map(self.width, self.height);
        let open = |h| self.tile(h).is_some_and(|t| entry.can_enter(self, t.owner));
        let exits = (!open(from)).then(|| self.exit_distances(map, open));
        crate::path::find_path_by_step(map, from, to, |a, b| {
            map.move_cost(b)
                .filter(|_| crate::path::border_step(map, a, b, open, exits.as_deref()))
        })
    }

    /// Factions are public information, including the owners of remembered cities.
    pub fn faction_of(&self, owner: PlayerId) -> Faction {
        if owner.is_barbarian() {
            return Faction::India;
        }
        self.player_public(owner).expect("known player").faction
    }

    /// The faction whose version of `u` it is: its seller for a hired Company, else its owner's
    /// (`GameState::made_by`). Origins are public, like factions.
    pub fn made_by(&self, u: &UnitView) -> Faction {
        u.origin.unwrap_or_else(|| self.faction_of(u.owner))
    }

    /// Barbarians never attack the Celts (`rules::barbarians::spares`): true when the viewer is
    /// the Celts, whose units, cities and civilians barbarians leave alone. Public rule.
    pub fn barbarians_spare_me(&self) -> bool {
        self.faction_of(self.me) == Faction::Celts
            && !self
                .rule_experiments
                .rule_off(crate::experiments::FactionRule::CeltsBarbarianPeace)
    }

    /// Whether `other` is a threat to `observer` (`rules::is_threat`): at War, except barbarians
    /// toward the Celts, who are spared. Automatic fire, wake-ups, alarms and warnings ask this;
    /// attack orders ask `at_war`. Public rules only.
    pub fn is_threat(&self, observer: PlayerId, other: PlayerId) -> bool {
        self.at_war(observer, other)
            && !(other.is_barbarian()
                && self.faction_of(observer) == Faction::Celts
                && !self
                    .rule_experiments
                    .rule_off(crate::experiments::FactionRule::CeltsBarbarianPeace))
    }

    /// An empire's era, 0-based (0 Ancient). Public. Barbarians are in the first era.
    pub fn era_of(&self, owner: PlayerId) -> u8 {
        if owner.is_barbarian() {
            return crate::eras::FIRST_ERA;
        }
        self.player_public(owner)
            .map_or(crate::eras::FIRST_ERA, |p| p.era.saturating_sub(1))
    }

    pub fn index(&self, h: Hex) -> Option<usize> {
        let (c, r) = h.to_offset();
        if c < 0 || r < 0 || c >= self.width || r >= self.height {
            None
        } else {
            Some((r * self.width + c) as usize)
        }
    }

    pub fn tile(&self, h: Hex) -> Option<&TileView> {
        self.index(h).map(|i| &self.tiles[i])
    }
}

/// `PlayerView::passage_allowed` for one mover and kind, answered from a table for every owner the view knows
/// (players and barbarians); any other owner falls back to the view's own rule.
#[derive(Clone, Copy, Debug)]
pub struct EntryTable {
    player: PlayerId,
    kind: UnitKind,
    open: [Option<bool>; 256],
}

impl EntryTable {
    pub fn can_enter(&self, view: &PlayerView, owner: Option<PlayerId>) -> bool {
        match owner {
            None => true,
            Some(o) => self.open[o.0 as usize]
                .unwrap_or_else(|| view.passage_allowed(self.player, self.kind, owner)),
        }
    }
}

/// `ceil(amount / per_turn)`, at least 1. Already funded work resolves next turn even at zero
/// income; callers must handle blockers such as resistance separately.
fn turns_needed(amount: i32, per_turn: i32) -> Option<i32> {
    if amount <= 0 {
        return Some(1);
    }
    if per_turn <= 0 {
        return None;
    }
    Some(((amount + per_turn - 1) / per_turn).max(1))
}

/// Every production item, in a fixed order: units, then buildings.
fn all_items() -> impl Iterator<Item = ProductionItem> {
    UnitKind::ALL
        .into_iter()
        .map(|unit| ProductionItem::Unit { unit })
        .chain(
            BuildingKind::ALL
                .into_iter()
                .filter(|building| !crate::landmarks::is_landmark(*building))
                .map(|building| ProductionItem::Building { building }),
        )
        .chain(
            [
                IndustryProject::Gold,
                IndustryProject::Science,
                IndustryProject::Food,
                IndustryProject::Grow,
            ]
            .map(|project| ProductionItem::Project { project }),
        )
}

/// The improvement a viewer sees on a tile: the live one while visible, else the remembered one.
/// Only a tile's own owner sees it in its upgraded form (which would reveal their technology).
fn improvement_view(
    state: &GameState,
    me: PlayerId,
    t: &crate::state::Tile,
    seen: Option<bool>,
) -> Option<TileImprovement> {
    if seen != Some(true) {
        return None;
    }
    let kind = crate::improvements::kind_for(t.resource?);
    let upgraded = t.owner == Some(me)
        && crate::improvements::upgraded(kind, |tech| state.player(me).techs.contains(&tech));
    Some(TileImprovement { kind, upgraded })
}

/// The preview shown with an Improvement production option.
fn improvement_preview(state: &GameState, c: &City, tile: Hex) -> Option<ImprovementPreview> {
    let resource = state.map.get(tile)?.resource?;
    let techs = &state.player(c.owner).techs;
    let knows = |t: TechId| techs.contains(&t);
    let kind = crate::improvements::kind_for(resource);
    let upgrade_tech = crate::improvements::upgrade_tech(kind);
    Some(ImprovementPreview {
        tile,
        resource,
        gain: crate::improvements::resource_yields(resource, knows),
        upgrade: (!knows(upgrade_tech)).then(|| ImprovementUpgrade {
            tech: upgrade_tech,
            name: crate::improvements::name(kind, true).to_string(),
            gain: crate::improvements::upgrade_yields(resource),
        }),
    })
}

/// Tiles this city may buy: border candidates within its work radius that its owner has
/// explored, nearest ring first (then hex order), with their prices.
fn buyable_tiles(state: &GameState, c: &City, buy_candidates: &[Hex]) -> Vec<BuyableTile> {
    let explored = &state.player(c.owner).explored;
    let mut tiles: Vec<BuyableTile> = buy_candidates
        .iter()
        .copied()
        .filter(|h| state.map.index(*h).is_some_and(|i| explored.get(i)))
        .map(|tile| BuyableTile {
            tile,
            ring: tile.distance(c.pos),
            price: crate::rules::cities::tile_price(c, tile),
        })
        .collect();
    tiles.sort_by_key(|b| (b.ring, b.tile));
    tiles
}

/// Build the view for `me`: fog applied, teammates share vision. Everything is ordered by id
/// (tiles by map index), so the view is deterministic.
/// The parts of every seat's view that don't depend on the viewer: players' public records,
/// titles and standings. Build them once per state with `public_view` and pass
/// them to `player_view_with` when building several seats' views of one state.
#[derive(Clone, Debug)]
pub struct PublicView {
    players: Shared<Vec<PlayerPublic>>,
    eras: Shared<Vec<EraView>>,
    standings: Shared<Vec<TeamStanding>>,
}

pub fn public_view(state: &GameState) -> PublicView {
    let players = state
        .players
        .iter()
        .map(|p| PlayerPublic {
            id: p.id,
            name: p.name.clone(),
            faction: p.faction,
            team: p.team,
            is_human: p.is_human,
            difficulty: state.setup.players[p.id.0 as usize].difficulty,
            search: state.setup.players[p.id.0 as usize].search,
            alive: p.alive,
            vp: scoring::vp_total(state, p.id),
            vp_sources: scoring::vp(state, p.id),
            capitals_taken: scoring::capitals_held(state, p.id),
            capitals_sacked: p.capitals_sacked.clone(),
            capital_stands: scoring::capital_stands(state, p.id),
            cities: state.cities_of(p.id).count() as u32,
            faction_goal_done: p.faction_goal_done,
            sakoku_turns: p.sakoku_turns,
            tomb_work: p.tomb_work,
            tomb_turns: p.tomb_turns,
            goal_rule: Some(crate::goals::rule(state, p.id)),
            era: objectives::era_of(state, p.id),
            gold: if p.alive { p.gold } else { 0 },
            economy: if p.alive {
                scoring::economy_value(state, p.id)
            } else {
                0
            },
            army: if p.alive {
                scoring::army_strength(state, p.id)
            } else {
                0
            },
            advancement: if p.alive {
                scoring::advancement(state, p.id)
            } else {
                0
            },
            levy: economy::levy(state, p.id).map(|(units, cap)| LevyView {
                units: units as u32,
                cap: cap as u32,
            }),
            companies: crate::company::count(state, p.id) as u32,
            power: scoring::power(state, p.id),
            lead_percent: scoring::lead_percent(state, p.id),
            decadence_percent: scoring::decadence_percent(state, p.id),
        })
        .collect();

    let eras = era_views(state);
    let standings = state
        .teams()
        .into_iter()
        .map(|t| TeamStanding {
            team: t,
            members: state.team_members(t),
            vp: scoring::team_vp(state, t),
            target: scoring::team_target(state, t),
            victory_hold: state.vp_holds.get(&t).map(|held| VictoryHold {
                turns_held: *held,
                turns_needed: crate::data::VP_HOLD_TURNS,
            }),
        })
        .collect();

    PublicView {
        players,
        eras: eras.into(),
        standings,
    }
}

/// Public objectives of every era after the first, with who has entered and scored each.
fn era_views(state: &GameState) -> Vec<EraView> {
    (2..=data::era_count())
        .map(|era| EraView {
            era,
            name: data::era_name(era).to_string(),
            objectives: objectives::era_objectives(&state.setup, era)
                .into_iter()
                .map(|o| ObjectiveView {
                    slot: objectives::slot_in(&state.setup, era, o),
                    family: o.family(),
                    text: o.text(),
                    target: o.target(),
                    objective: o,
                })
                .collect(),
            entered_by: state
                .players
                .iter()
                .filter(|p| p.alive && objectives::era_of(state, p.id) >= era)
                .map(|p| p.id)
                .collect(),
            scored_by: state
                .players
                .iter()
                .filter_map(|p| {
                    let s = p.eras.get(&era)?.scored?;
                    Some(EraScoredView {
                        player: p.id,
                        slot: s.slot,
                        turn: s.turn,
                    })
                })
                .collect(),
        })
        .collect()
}

/// The viewer's progress on every objective of the eras it has entered and the next one.
fn objective_progress(state: &GameState, me: PlayerId) -> Vec<ObjectiveProgress> {
    let player = state.player(me);
    (2..=objectives::revealed_era(state))
        .flat_map(|era| {
            objectives::era_objectives(&state.setup, era)
                .into_iter()
                .map(move |o| ObjectiveProgress {
                    era,
                    slot: objectives::slot_in(&state.setup, era, o),
                    current: o.progress(state, me, era),
                    target: o.target(),
                    entered: player.eras.contains_key(&era),
                    era_scored: player.eras.get(&era).is_some_and(|r| r.scored.is_some()),
                    entered_turn: player.eras.get(&era).map(|r| r.entered),
                    ready: Some(o.met(state, me, era)),
                })
        })
        .collect()
}

fn objective_accounting(state: &GameState, me: PlayerId) -> Vec<ObjectiveAccounting> {
    let player = state.player(me);
    (2..=objectives::era_of(state, me).max(objectives::revealed_era(state)))
        .flat_map(|era| {
            let Some(record) = player.eras.get(&era) else {
                return Vec::new();
            };
            let restart_eras: Vec<u8> = if record.scored.is_none() {
                player
                    .eras
                    .iter()
                    .filter_map(|(&other, r)| (other != era && r.scored.is_none()).then_some(other))
                    .collect()
            } else {
                Vec::new()
            };
            objectives::era_objectives(&state.setup, era)
                .into_iter()
                .filter_map(|o| {
                    let (baseline, total) = o.count_accounting(state, me, era)?;
                    Some(ObjectiveAccounting {
                        era,
                        slot: objectives::slot_in(&state.setup, era, o),
                        baseline,
                        total,
                        counting_since_turn: record.counting_since,
                        restart_eras: restart_eras.clone(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// `ViewDetail::Map` for the engine's order checks. Players carry only identity, faction and
/// team (the checks read nothing else), so no scoring runs.
pub(crate) fn order_check_view(state: &GameState, me: PlayerId) -> PlayerView {
    let public = PublicView {
        players: state
            .players
            .iter()
            .map(|p| PlayerPublic {
                id: p.id,
                name: String::new(),
                faction: p.faction,
                team: p.team,
                is_human: p.is_human,
                difficulty: state.setup.players[p.id.0 as usize].difficulty,
                search: state.setup.players[p.id.0 as usize].search,
                alive: p.alive,
                vp: 0,
                vp_sources: Vec::new(),
                capitals_taken: Vec::new(),
                capitals_sacked: Vec::new(),
                capital_stands: true,
                cities: 0,
                faction_goal_done: p.faction_goal_done,
                sakoku_turns: p.sakoku_turns,
                tomb_work: p.tomb_work,
                tomb_turns: p.tomb_turns,
                goal_rule: Some(crate::goals::rule(state, p.id)),
                era: objectives::era_of(state, p.id),
                gold: 0,
                economy: 0,
                army: 0,
                advancement: 0,
                levy: None,
                companies: 0,
                power: 0,
                lead_percent: 0,
                decadence_percent: 0,
            })
            .collect(),
        eras: Default::default(),
        standings: Default::default(),
    };
    player_view_detail(state, me, &public, ViewDetail::Map)
}

pub fn player_view(state: &GameState, me: PlayerId) -> PlayerView {
    player_view_with(state, me, &public_view(state))
}

/// `player_view`, with the viewer-independent parts from `public_view(state)`.
pub fn player_view_with(state: &GameState, me: PlayerId, public: &PublicView) -> PlayerView {
    player_view_detail(state, me, public, ViewDetail::Full)
}

/// How much of a view to build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewDetail {
    /// Everything a human sees: labelled breakdowns, previews and forecasts.
    Full,
    /// Only what AI decisions read (the AI seats of a game, simulations). Every field the AI
    /// reads is identical to `Full`; the rest is left empty:
    /// - `CityYields::sources` and `focus_reason` (labels) in city details, build previews
    ///   and tech unlocks (totals, worked tiles and focus are kept);
    /// - `UnlockCity`'s growth forecasts (`turns_to_six_after`, `turns_to_eight_after`,
    ///   `turns_to_eight_at_gate`), and its `yields_after` (`CityYields::empty`) and
    ///   `food_surplus_after` (0) except for the Forge and Factory, whose Production
    ///   gain the AI prices;
    /// - `TechOption::income_change` and `TechEffects::growth`;
    /// - build options' `buy_quote` and `upkeep` previews, and `CityDetail::population_unlocks`;
    /// - the words of blocks the AI only checks for (`ProductionOption::buy_blocker` and
    ///   `resource_blocker`, `UnavailableOption::reason`): set
    ///   exactly when a full view sets them, but empty, so nothing is formatted;
    /// - `CityDetail::buyable_tiles` and `buy_tile_blocker` (the AI never buys tiles).
    ///
    /// Adding a field to this list needs proof that no brain reads it (`legatum_ai`, the
    /// search AI included); `sim_bench` digests and `tools/sim/repin.sh` must not move.
    Decisions,
    /// The map as the viewer knows it, for the engine's order checks
    /// (`rules::units::check_known_path` and friends): tiles, units, cities without detail,
    /// players, diplomacy and the viewer. Everything else is empty. Never given to a brain.
    Map,
}

/// `player_view_with` at the given level of detail.
pub fn player_view_detail(
    state: &GameState,
    me: PlayerId,
    public: &PublicView,
    detail: ViewDetail,
) -> PlayerView {
    player_view_cached(
        state,
        me,
        public,
        detail,
        &mut crate::yields_cache::ViewCache::default(),
    )
}

/// `player_view_detail`, reusing city yields from earlier builds in `cache` while their inputs
/// are unchanged (`yields_cache`). The view is identical either way; keep one cache per game.
pub fn player_view_cached(
    state: &GameState,
    me: PlayerId,
    public: &PublicView,
    detail: ViewDetail,
    cache: &mut crate::yields_cache::ViewCache,
) -> PlayerView {
    cache.begin(state);
    let full = detail == ViewDetail::Full;
    let map_only = detail == ViewDetail::Map;
    let player = state.player(me);
    let income = (!map_only).then(|| economy::player_income(state, me));
    let mut forecasts = income
        .as_ref()
        .map(|income| forecast::forecast_cities(state, me, income));
    let military_units = forecasts.as_ref().map_or(0, |f| f.military_units);
    let gold_per_turn = income.as_ref().map_or(0, |i| i.gold.total);
    let supply = (!map_only).then(|| crate::resources::ViewSupply::new(state, me));
    let team = player.team;
    let visible = visibility::visible_tiles(state, team);

    let tiles: Vec<TileView> = state
        .map
        .tiles
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let vis = visible.get(i);
            let memory = player.tile_memory.get(i).and_then(Option::as_ref);
            let explored = vis || player.explored.get(i) || memory.is_some();
            TileView {
                explored,
                visible: vis,
                terrain: explored.then_some(t.terrain),
                ruin: if vis {
                    t.ruin
                } else {
                    explored && memory.is_some_and(|m| m.ruin)
                },
                wonder: if explored { t.wonder } else { None },
                landmark: t.landmark,
                wonder_found: explored
                    && t.wonder.is_some()
                    && state.wonders_discovered.contains(&state.map.hex_at(i)),
                owner: if explored || t.landmark.is_some() {
                    t.owner
                } else {
                    None
                },
                resource: crate::resources::visible_resource(t, explored, &player.techs),
                improvement: improvement_view(
                    state,
                    me,
                    t,
                    if vis {
                        Some(t.improved)
                    } else {
                        memory.filter(|_| explored).map(|m| m.improved)
                    },
                ),
                road: if explored { t.road } else { 0 },
            }
        })
        .collect();
    let city_gap_blocked = state
        .cities
        .values()
        .flat_map(|city| {
            city.pos
                .within_iter(state.setup.rule_experiments.min_city_gap(player.faction))
        })
        .filter(|&pos| {
            state.map.index(pos).is_some_and(|i| {
                let tile = &tiles[i];
                tile.explored
                    && tile.wonder.is_none()
                    && tile
                        .terrain
                        .is_some_and(|t| t.is_land() && t.move_cost().is_some())
                    && tile.owner.is_none_or(|owner| owner == me)
            })
        })
        .collect();
    let is_visible = |h: Hex| state.map.index(h).is_some_and(|i| visible.get(i));

    let units = state
        .units
        .values()
        .filter(|u| {
            state.team_of(u.owner) == team
                || is_visible(u.pos)
                || visibility::attack_revealed_to(state, team, u)
                // The Heir's position is public to every empire, in fog too (user, 2026-10-22).
                || u.kind == UnitKind::HeirToTheThrone
        })
        .map(|u| unit_view(state, me, u))
        .collect();

    // Territory of the viewer's own cities, in map order, gathered in one pass.
    let mut territory: BTreeMap<CityId, Vec<Hex>> = BTreeMap::new();
    for (i, t) in state.map.tiles.iter().enumerate() {
        if let Some(cid) = t.city
            && state.cities.get(&cid).is_some_and(|c| c.owner == me)
        {
            territory.entry(cid).or_default().push(state.map.hex_at(i));
        }
    }

    let mut cities: Vec<CityView> = state
        .cities
        .values()
        .filter(|c| state.map.index(c.pos).is_some_and(|i| tiles[i].explored))
        .map(|c| {
            let vis = is_visible(c.pos);
            let memory = state.map.index(c.pos).and_then(|i| {
                player
                    .tile_memory
                    .get(i)
                    .and_then(Option::as_ref)
                    .and_then(|m| m.city.as_ref())
            });
            let current = CityMemory::from(c);
            let observation = if vis {
                current
            } else {
                CityMemory::with_public_identity(&current, memory)
            };
            let mut observed = CityView::observed(
                &observation,
                c.pos,
                vis,
                forecasts
                    .as_mut()
                    .filter(|_| vis && c.owner == me)
                    .map(|forecasts| {
                        city_detail(
                            state,
                            c,
                            territory.remove(&c.id).unwrap_or_default(),
                            forecasts.production_costs.remove(&c.id),
                            forecasts.stalls.remove(&c.id),
                            military_units,
                            gold_per_turn,
                            detail,
                            supply.as_ref().expect("city detail needs supply"),
                            cache,
                        )
                    }),
                state.player(c.owner).faction,
                state.setup.rule_experiments,
                state.era_of(c.owner),
            );
            observed.great_works = c
                .buildings
                .iter()
                .copied()
                .filter(|building| data::is_great_work(*building))
                .collect();
            observed.occupation_lock = (vis || c.owner == me)
                .then(|| crate::rules::occupation::entering_lock(state, me, c))
                .flatten();
            observed.original_capital = c.original_capital;
            observed.original_capital_of = observed.original_capital.then_some(c.founder);
            observed.founder = Some(c.founder);
            observed.greece_captive_tech =
                crate::rules::battle::greece_captive_available(state, c.id, me);
            observed.greece_captive_at_risk = c.owner == me
                && state.player(me).faction == Faction::Athens
                && state.players.iter().any(|r| {
                    r.alive && crate::rules::battle::greece_captive_available(state, c.id, r.id)
                });
            observed.walls_building = matches!(
                c.producing,
                Some(ProductionItem::Building {
                    building: BuildingKind::Walls
                })
            );
            if let Some(ProductionItem::Wonder { building, tile }) = c.producing {
                let cost = economy::city_production_cost(state, c, c.producing.unwrap()).max(1);
                let progress = ((c.production_stored.max(0) as i64 * 1000) / cost as i64)
                    .clamp(0, 1000) as u16;
                observed.construction = Some(ConstructionView {
                    building,
                    tile,
                    stage: if progress >= 750 {
                        3
                    } else if progress >= 400 {
                        2
                    } else if progress >= 100 {
                        1
                    } else {
                        0
                    },
                    progress,
                });
            }
            if vis && state.player(c.owner).faction == Faction::Switzerland {
                observed.capture_bounty = Some(crate::economy::swiss_capture_bounty(state, c));
            }
            observed
        })
        .collect();
    cities.sort_by_key(|c| c.id);

    let (Some(income), Some(forecasts)) = (income, forecasts) else {
        return PlayerView {
            rule_experiments: state.setup.rule_experiments,
            ai_alliances: !state.setup.ai_alliance_pairs().is_empty(),
            diplomacy: crate::diplomacy::view(state, me).into(),
            turn: state.turn,
            turn_limit: state.setup.turn_limit,
            me,
            team,
            width: state.map.width,
            height: state.map.height,
            tiles: tiles.into(),
            units,
            cities: cities.into(),
            failed_city_sites: Default::default(),
            city_gap_blocked,
            observed_threats: Vec::new(),
            barbarian_camps: Vec::new(),
            players: public.players.clone(),
            economy: Default::default(),
            supply: Vec::new(),
            research: Default::default(),
            army_breakdown: Vec::new(),
            eras: Default::default(),
            objective_progress: Vec::new(),
            objective_accounting: Vec::new(),
            development: None,
            faction_goal: None,
            actions: Default::default(),
            courier_halls: crate::goals::courier_halls(state),
            imperial_diwans: crate::goals::imperial_diwans(state),
            colonias: crate::goals::colonias(state),
            royal_tombs: crate::goals::royal_tombs(state),
            tomb_sites: crate::goals::tomb_sites(state),
            aircraft: aircraft_views(state, team),
            air_sightings: Vec::new(),
            air_contacts: Vec::new(),
            company_offers: Vec::new(),
            great_works: Vec::new(),
            standings: Default::default(),
            winner: state.winner,
            finished: state.finished,
            history: None,
        };
    };

    let PublicView {
        players,
        eras,
        standings,
    } = public.clone();
    // A future Age's card stays hidden from every view until an empire enters it.
    let eras = eras
        .iter()
        .filter(|e| e.era <= objectives::revealed_era(state))
        .cloned()
        .collect::<Vec<_>>()
        .into();

    let science = income.science.total;
    let economy = EconomyView {
        gold: player.gold,
        military_purchases: player.military_purchases,
        gold_per_turn: income.gold,
        science_per_turn: income.science,
        upkeep: income.upkeep,
        upkeep_breakdown: economy::upkeep_breakdown(state, me),
        bankruptcy: forecasts.bankruptcy,
    };

    // Stored science carries over when switching, so it counts toward every option.
    let stored = player.science_stored;
    let current_cost = player
        .researching
        .map(|t| economy::tech_cost(state, me, t).total);
    let option = |tech: TechId| {
        let cost = economy::tech_cost(state, me, tech);
        TechOption {
            tech,
            known_by: economy::tech_known_by(state, me, tech),
            turns: turns_needed(cost.total - stored, science),
            cost,
            era: crate::eras::era_of_tech(tech),
            researchable: economy::tech_available(state, me, tech),
            income_change: if full {
                crate::research::income_change(state, me, tech)
            } else {
                Vec::new()
            },
            effects: crate::research::tech_effects_for(state, me, tech),
        }
    };
    let queue: Vec<TechId> = if player.research_queue.is_empty() {
        player.researching.into_iter().collect()
    } else {
        player.research_queue.clone()
    };
    let mut schedule = Vec::new();
    let mut cumulative = 0;
    let mut previous: Option<i32> = None;
    for tech in &queue {
        let cost = economy::tech_cost(state, me, *tech).total;
        cumulative += cost;
        // The engine completes one technology per turn, so each arrives at least a turn after
        // the one before it however much Science is stored.
        let turns = turns_needed(cumulative - stored, science)
            .map(|t| previous.map_or(t, |p: i32| t.max(p + 1)));
        previous = turns;
        schedule.push(ScheduleSlot {
            tech: *tech,
            cost,
            turns,
            arrival_turn: turns.map(|t| state.turn + t as u32),
        });
    }
    let era = crate::eras::current_era(&player.techs);
    let skipped: Vec<TechId> =
        crate::data::skipped_techs(state.setup.rule_experiments, player.faction).collect();
    let era_techs = || crate::eras::era_techs(era).filter(|t| !skipped.contains(t));
    let research = ResearchView {
        science_conversion: player.science_conversion,
        era,
        era_name: crate::eras::era_name(era).into(),
        era_total: era_techs().count() as u32,
        era_known: era_techs().filter(|t| player.techs.contains(t)).count() as u32,
        era_firsts: state.era_firsts.iter().map(|(e, p)| (*e, *p)).collect(),
        schedule,
        current: player.researching,
        queue,
        progress: stored,
        cost: current_cost,
        turns_left: current_cost.and_then(|cost| turns_needed(cost - stored, science)),
        known: player.techs.iter().copied().collect(),
        skipped: skipped.clone(),
        catalog: TechId::ALL.into_iter().map(option).collect(),
        unlocks: crate::research::tech_unlocks_for(state, me, full, cache),
        available: TechId::ALL
            .into_iter()
            .filter(|t| economy::tech_available(state, me, *t))
            .map(option)
            .collect(),
    };

    PlayerView {
        rule_experiments: state.setup.rule_experiments,
        ai_alliances: !state.setup.ai_alliance_pairs().is_empty(),
        diplomacy: crate::diplomacy::view(state, me).into(),
        turn: state.turn,
        turn_limit: state.setup.turn_limit,
        me,
        team,
        width: state.map.width,
        height: state.map.height,
        tiles: tiles.into(),
        units,
        cities: cities.into(),
        failed_city_sites: player.failed_city_sites.clone(),
        city_gap_blocked,
        observed_threats: visibility::threat_observations(state, me, &visible),
        barbarian_camps: state
            .barbarians
            .as_ref()
            .and_then(|b| b.known.get(&team))
            .into_iter()
            .flatten()
            .map(|&pos| BarbarianCampView {
                pos,
                visible: is_visible(pos),
            })
            .collect(),
        players,
        economy: economy.into(),
        supply: crate::resources::supply_views(state, me),
        research: research.into(),
        army_breakdown: if full {
            crate::army::breakdown(state, me)
        } else {
            Vec::new()
        },
        eras,
        objective_progress: objective_progress(state, me),
        objective_accounting: objective_accounting(state, me),
        development: Some(crate::projects::development_forecast_cached(
            state, me, cache,
        )),
        faction_goal: crate::goals::view(state, me),
        actions: crate::rules::actions::view(state, me),
        courier_halls: crate::goals::courier_halls(state),
        imperial_diwans: crate::goals::imperial_diwans(state),
        colonias: crate::goals::colonias(state),
        royal_tombs: crate::goals::royal_tombs(state),
        tomb_sites: crate::goals::tomb_sites(state),
        aircraft: aircraft_views(state, team),
        air_sightings: state
            .aircraft
            .iter()
            .filter_map(|a| {
                let city = state.cities.get(&a.city)?;
                (state.team_of(city.owner) != team && is_visible(city.pos)).then_some(AirSighting {
                    owner: city.owner,
                    kind: a.kind,
                    city: a.city,
                })
            })
            .collect(),
        air_contacts: state.air_contacts.get(&me).cloned().unwrap_or_default(),
        company_offers: crate::company::offers(state, me),
        great_works: crate::projects::great_work_views(state, me),
        standings,
        winner: state.winner,
        finished: state.finished,
        history: (state.finished && full).then(|| state.history.clone()),
    }
}

fn unit_view(state: &GameState, me: PlayerId, u: &Unit) -> UnitView {
    let stats = state.unit_stats_of(u);
    let own = u.owner == me;
    UnitView {
        id: u.id,
        owner: u.owner,
        kind: u.kind,
        pos: u.pos,
        hp: u.hp,
        moves_left: u.moves_left,
        max_moves: state.unit_moves_of(u),
        goto: if own { u.goto } else { None },
        path: if own { u.path.clone() } else { Vec::new() },
        automation: if own { u.automation } else { None },
        standing_order: if own { u.standing_order } else { None },
        needs_orders: own && u.needs_orders,
        attack_target: if own { u.attack_target } else { None },
        fortify: u.fortify,
        created_turn: own.then_some(u.created_turn),
        strength: stats.strength,
        ranged: stats.ranged,
        range: stats.range,
        military: stats.military,
        level: crate::rules::promotion::level_for_rules(
            state.setup.rule_experiments.faction_rules_off,
            state.player(u.owner).faction,
            u.xp,
        ),
        xp: (own && stats.military).then_some(u.xp),
        xp_next: if own && stats.military {
            crate::rules::promotion::next_level_xp_for_rules(
                state.setup.rule_experiments.faction_rules_off,
                state.player(u.owner).faction,
                u.xp,
            )
        } else {
            None
        },
        occupation_lock: own
            .then(|| crate::rules::occupation::lock(state, u).map(|(_, from)| from))
            .flatten(),
        sight: own.then(|| {
            data::UNIT_SIGHT
                + i32::from(
                    !u.owner.is_barbarian()
                        && data::is_medjay_rules(
                            state.setup.rule_experiments.faction_rules_off,
                            state.made_by(u),
                            u.kind,
                        ),
                )
                + i32::from(
                    state
                        .map
                        .get(u.pos)
                        .is_some_and(|t| t.terrain == Terrain::Hills),
                )
        }),
        resting_heal: own.then(|| crate::rules::units::resting_heal(state, u)),
        heir_of: u.heir_of,
        contract_until: u.contract_until,
        origin: u.origin,
        upgrade_cost: if own { upgrade_cost(state, u) } else { None },
        upgrade_to: own
            .then(|| crate::rules::units::upgrade_plan(state, u.owner, u.kind).map(|plan| plan.0))
            .flatten(),
        upgrade_steps: if own {
            crate::rules::units::upgrade_plan(state, u.owner, u.kind)
                .map(|plan| plan.1)
                .unwrap_or_default()
        } else {
            Vec::new()
        },
        upgrade_breakdown: own
            .then(|| crate::rules::units::upgrade_plan(state, u.owner, u.kind).map(|plan| plan.2))
            .flatten(),
        upgrade_need: own
            .then(|| {
                crate::rules::units::upgrade_plan(state, u.owner, u.kind)
                    .and_then(|plan| crate::resources::unit_upgrade_need_to(u, plan.0))
            })
            .flatten(),

        unsupplied: crate::resources::unit_unsupplied(state, u),
        unsupplied_loss: crate::resources::unit_unsupplied(state, u)
            .map(|_| crate::resources::unsupplied_penalty(stats.strength)),
        upgrade_supply_blocker: own
            .then(|| {
                crate::rules::units::upgrade_plan(state, u.owner, u.kind).and_then(|plan| {
                    crate::resources::upgrade_blocker(state, u.owner, u, plan.0, 0)
                })
            })
            .flatten(),
        siege: u.siege,
        siege_offer: (own
            && data::can_have_siege(u.kind)
            && !u.siege
            && data::siege_tier(&state.player(u.owner).techs) > 0)
            .then(|| {
                let gold = state.player(u.owner).gold;
                let blocker = crate::rules::siege::check(state, u.owner, u.id, gold, 0).err();
                SiegeOffer {
                    price: crate::rules::siege::price(state, u.owner, u.kind),
                    waits_for_gold: blocker.is_some()
                        && crate::rules::siege::check(state, u.owner, u.id, i32::MAX, 0).is_ok(),
                    blocker,
                }
            }),
    }
}

/// Gold to upgrade this unit now: in its owner's territory, with the tech, and affordable.
fn upgrade_cost(state: &GameState, u: &Unit) -> Option<i32> {
    crate::rules::units::check_upgrade(state, u.owner, u.id, state.player(u.owner).gold)
        .ok()
        .map(|(_, cost)| cost)
}

/// The upkeep effect of building one more of `item` now, given `military_units` (the player's
/// current upkeep-paying land unit count, including anything the core forecast has completing this
/// turn) and the player's actual current `gold_per_turn`. None for civilians, buildings and
/// projects, which have no upkeep.
fn unit_upkeep_preview(
    state: &GameState,
    c: &City,
    item: ProductionItem,
    military_units: i32,
    gold_per_turn: i32,
) -> Option<UpkeepPreview> {
    let ProductionItem::Unit { unit } = item else {
        return None;
    };
    if !data::pays_upkeep(unit) {
        return None;
    }
    let before = economy::upkeep_breakdown_with_military(state, c.owner, military_units).total;
    let after = economy::upkeep_breakdown_with_military(state, c.owner, military_units + 1).total;
    let delta = after - before;
    Some(UpkeepPreview {
        free: delta == 0,
        free_slots_left: (data::FREE_MILITARY_UNITS - military_units).max(0),
        rate: data::military_upkeep_rules(
            state.setup.rule_experiments.faction_rules_off,
            state.player(c.owner).faction,
        ),
        gold_per_turn_after: gold_per_turn - delta,
    })
}

/// A block's wording in a full view; in a decision view only that it is blocked (empty text):
/// the AI reads whether these are set, never their words.
fn worded(
    full: bool,
    words: impl FnOnce() -> Option<String>,
    blocked: impl FnOnce() -> Option<economy::Blocked>,
) -> Option<String> {
    if full {
        words()
    } else {
        blocked().map(|_| String::new())
    }
}

#[allow(clippy::too_many_arguments)]
fn city_detail(
    state: &GameState,
    c: &City,
    territory: Vec<Hex>,
    production_cost: Option<i32>,
    production_stall: Option<ProductionStall>,
    military_units: i32,
    gold_per_turn: i32,
    detail: ViewDetail,
    supply: &crate::resources::ViewSupply,
    cache: &mut crate::yields_cache::ViewCache,
) -> CityDetail {
    let production_cost = if matches!(c.producing, Some(ProductionItem::Project { .. })) {
        None
    } else {
        production_cost
    };
    // Builds change only the build-dependent lines (focus, Germany's industry projects, the
    // Nile, decadence): derive every preview from the build-independent base.
    let full = detail == ViewDetail::Full;
    let mut cached = cache.take(state, c);
    let base = cached.base(state, c, full);
    cache.put(c.id, cached);
    let yields = economy::city_yields_for_item(state, c, c.producing, &base, full);
    let claims = crate::rules::cities::claim_candidates_from(state, c, &territory);
    // Only a human's panel and map show prices; the AI never buys tiles, so a decision view
    // skips the list (the territory walk above is shared with the next claim).
    let buyable = if full {
        buyable_tiles(
            state,
            c,
            &crate::rules::cities::buy_candidates_among(c, &claims),
        )
    } else {
        Vec::new()
    };
    let growth_threshold = economy::growth_threshold(state, c).total;
    let surplus = yields.total.food - FOOD_PER_POP * c.population;
    let production = yields.total.production;
    let idle_production_cap = crate::rules::cities::bank_cap(state, c);
    let idle_production_waste = if c.producing.is_none() && c.resistance == 0 {
        (production - (idle_production_cap - c.production_stored).max(0)).max(0)
    } else {
        0
    };
    let production_turns = |remaining| {
        if c.resistance > 0 {
            None
        } else {
            let rate = if c
                .producing
                .is_some_and(|item| crate::rules::cities::egypt_building(state, c, item))
            {
                surplus.max(0).saturating_mul(2)
            } else {
                production
            };
            turns_needed(remaining, rate)
        }
    };
    // Items whose only missing requirement is population: they could continue at their
    // minimum size (a Settler's population gate also applies while it is built).
    let mut population_unlocks: Vec<PopulationUnlock> = all_items()
        .filter(|_| full)
        .filter(|item| item_min_population(*item) > 1)
        .filter(|item| {
            let mut grown = c.clone();
            grown.population = grown.population.max(item_min_population(*item));
            economy::can_continue_production(state, &grown, *item)
        })
        .map(|item| PopulationUnlock {
            population: item_min_population(item),
            item,
        })
        .collect();
    population_unlocks.sort_by_key(|u| u.population);
    let era = state.era_of(c.owner);
    // The Nile: Egypt has no Production, so a Quarry (Production only) would give it nothing.
    // The order stays legal; the view just doesn't offer it (no rules change).
    let nile = state.player(c.owner).faction == Faction::Egypt
        && state.setup.rule_experiments.egypt_nile();
    let options = all_items()
        .chain(
            crate::improvements::candidates(state, c)
                .into_iter()
                .filter(|item| {
                    !(nile
                        && matches!(
                            item,
                            ProductionItem::Improvement {
                                improvement: ImprovementKind::Quarry,
                                ..
                            }
                        ))
                }),
        )
        .chain(
            BuildingKind::ALL
                .into_iter()
                .filter(|b| crate::landmarks::is_landmark(*b))
                .flat_map(|b| crate::landmarks::candidates(state, c, b)),
        )
        .filter(|item| {
            economy::production_check_at_era::<economy::Blocked>(state, c, *item, true, era)
                .is_none()
        })
        .map(|item| {
            let cost = economy::city_production_cost(state, c, item);
            let egypt = crate::rules::cities::egypt_building(state, c, item);
            let stored = if egypt && c.producing != Some(item) {
                c.banked(item)
            } else {
                c.production_for(item)
            };
            let buy_cost = economy::buy_cost_for(state, c, item);
            // Decision views keep only whether buying is blocked (the AI reads `buy_block`).
            let (buy_block, buy_blocker) = if full {
                economy::buy_check_with_supply::<String>(state, c, item, buy_cost, Some(supply))
                    .map_or((None, None), |(b, r)| (Some(b), Some(r)))
            } else {
                economy::buy_check_with_supply::<economy::Blocked>(
                    state,
                    c,
                    item,
                    buy_cost,
                    Some(supply),
                )
                .map_or((None, None), |(b, _)| (Some(b), Some(String::new())))
            };
            let preview = economy::city_yields_for_item(state, c, Some(item), &base, full);
            ProductionOption {
                item,
                cost,
                production_stored: stored,
                production_banked: if c.producing == Some(item) {
                    0
                } else {
                    c.banked(item)
                },
                buy_cost,
                buy_blocker,
                buy_block,
                buy_quote: full.then(|| economy::buy_quote(state, c, item)).flatten(),
                turns: if matches!(item, ProductionItem::Project { .. }) || c.resistance > 0 {
                    -1
                } else {
                    turns_needed(
                        cost - stored,
                        if egypt {
                            (preview.total.food - FOOD_PER_POP * c.population)
                                .max(0)
                                .saturating_mul(2)
                        } else {
                            preview.total.production
                        },
                    )
                    .unwrap_or(-1)
                },
                yields: preview,
                supply_need: crate::resources::item_need_for(state.player(c.owner), item),
                resource_blocker: (c.producing != Some(item))
                    .then(|| {
                        worded(
                            full,
                            || {
                                crate::resources::start_check_with_supply(
                                    state,
                                    c,
                                    item,
                                    Some(supply),
                                )
                            },
                            || {
                                crate::resources::start_check_with_supply(
                                    state,
                                    c,
                                    item,
                                    Some(supply),
                                )
                            },
                        )
                    })
                    .flatten(),
                upkeep: full
                    .then(|| unit_upkeep_preview(state, c, item, military_units, gold_per_turn))
                    .flatten(),
                improvement: match item {
                    ProductionItem::Improvement { tile, .. } => improvement_preview(state, c, tile),
                    _ => None,
                },
                wonder: match item {
                    ProductionItem::Wonder { tile, .. } => {
                        state.map.get(tile).map(|t| WonderPreview {
                            tile,
                            lost: economy::owner_tile_yields(state, c.owner, t, None),
                        })
                    }
                    _ => None,
                },
            }
        })
        .collect();
    let unavailable = all_items()
        .filter(|item| {
            (state.setup.rule_experiments.available_to_player(
                state.player(c.owner).faction,
                era,
                *item,
            ) || (state.player(c.owner).faction == Faction::Aztecs
                && matches!(item, ProductionItem::Unit { unit } if data::is_horse_mounted(*unit)))
                || (state.player(c.owner).faction == Faction::Mongols
                    && matches!(
                        item,
                        ProductionItem::Building {
                            building: BuildingKind::Walls
                                | BuildingKind::Castle
                                | BuildingKind::StarFort
                        }
                    )))
                && !matches!(item, ProductionItem::Project { .. })
                && !data::is_great_work_item(*item)
        })
        .filter(|item| {
            data::item_requires_for_rules(state.setup.rule_experiments, *item)
                .is_none_or(|t| state.player(c.owner).techs.contains(&t))
        })
        .filter(|item| {
            !matches!(item, ProductionItem::Building { building }
                if data::has_building(&c.buildings, *building))
        })
        .filter_map(|item| {
            worded(
                full,
                || economy::production_check_at_era(state, c, item, true, era),
                || economy::production_check_at_era(state, c, item, true, era),
            )
            .map(|reason| UnavailableOption {
                item,
                cost: economy::city_production_cost(state, c, item),
                prerequisite_building: match item {
                    ProductionItem::Building { building } => {
                        crate::data::building_predecessor(building)
                            .filter(|previous| !crate::data::has_building(&c.buildings, *previous))
                    }
                    _ => None,
                },
                reason,
                production_banked: c.banked(item),
            })
        })
        .collect();
    CityDetail {
        captured: c.founder != c.owner,
        original_capital: crate::rules::cities::is_original_capital(state, c),
        former_owner: (c.founder != c.owner).then(|| crate::rules::cities::former_owner(state, c)),
        raze_blocker: (c.founder != c.owner)
            .then(|| crate::rules::cities::raze_blocker(state, c.owner, c.id))
            .flatten(),
        idle_yields: Some(economy::city_yields_for_item(state, c, None, &base, full)),
        food_stored: c.food_stored,
        growth_threshold,
        growth_breakdown: economy::growth_threshold(state, c),
        turns_to_grow: if c.resistance > 0
            || c.producing
                .is_some_and(|item| crate::rules::cities::egypt_building(state, c, item))
                && economy::production_blocker(state, c, c.producing.expect("checked"), false)
                    .is_none()
        {
            None
        } else if c.food_stored + surplus >= growth_threshold {
            Some(1)
        } else if surplus > 0 {
            turns_needed(growth_threshold - c.food_stored, surplus)
        } else {
            None
        },
        producing: c.producing,
        production_queue: c.production_queue.clone(),
        wonder_lost: c.wonder_lost,
        build_automation: c.build_automation,
        last_auto_pick: c.last_auto_pick,
        auto_queued: c.auto_queued,
        production_stored: c.production_stored,
        production_cost,
        turns_to_complete: production_cost
            .filter(|_| {
                production_stall.is_none()
                    && c.producing
                        .is_none_or(|item| economy::can_continue_production(state, c, item))
            })
            .and_then(|cost| production_turns(cost - c.production_stored)),
        food_construction_paused: c
            .producing
            .filter(|item| crate::rules::cities::egypt_building(state, c, *item))
            .and_then(|item| economy::production_blocker(state, c, item, false)),
        production_stall,
        idle_production_cap,
        idle_production_waste,
        buy_cost: economy::buy_cost(state, c),
        buildings: c.buildings.iter().copied().collect(),

        idle_buildings: crate::resources::idle_tiers(state.player(c.owner), c)
            .into_iter()
            .map(|(building, resource)| IdleBuilding { building, resource })
            .collect(),
        options,
        unavailable,
        production_banked: c.production_banked.clone(),
        territory,
        next_claim: crate::rules::cities::next_claim_among(state, c, claims).map(|tile| {
            ClaimPreview {
                tile,
                yields: state.map.get(tile).map_or(Yields::ZERO, |t| {
                    economy::owner_tile_yields(state, c.owner, t, None)
                }),
                worked: tile.distance(c.pos) <= data::CITY_WORK_RADIUS,
            }
        }),
        tile_base_price: crate::rules::cities::tile_base_price(c),
        buy_tile_blocker: if full {
            crate::rules::cities::buy_tile_slot_check::<String>(state, c).or_else(|| {
                buyable.is_empty().then(|| {
                    format!(
                        "{} has no free tile to buy within {} tiles of the city.",
                        c.name,
                        data::CITY_WORK_RADIUS
                    )
                })
            })
        } else {
            None
        },
        buyable_tiles: buyable,
        resistance: c.resistance,
        turns_to_starve: (c.resistance == 0 && surplus < 0 && c.population > 1)
            .then(|| c.food_stored / -surplus + 1),
        turns_to_heal: (c.hp < data::CITY_HP)
            .then(|| (data::CITY_HP - c.hp + data::CITY_HEAL - 1) / data::CITY_HEAL),
        heal_blocked: c.hp < data::CITY_HP
            && crate::rules::cities::heal_blocked_by_adjacent_enemy(state, c),
        attacked_last_turn: c.last_attacked_turn.is_some_and(|t| t + 1 == state.turn),
        population_unlocks,
        yields,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::game::{blank_game, spawn_unit};
    use crate::state::{GameSetup, Map};
    use std::collections::BTreeSet;

    pub(crate) const P0: PlayerId = PlayerId(0);
    pub(crate) const P1: PlayerId = PlayerId(1);
    pub(crate) const P2: PlayerId = PlayerId(2);
    pub(crate) const P3: PlayerId = PlayerId(3);

    pub(crate) fn at(col: i32, row: i32) -> Hex {
        Hex::from_offset(col, row)
    }

    /// Standard 8-empire setup (P0 + P1 on team 0, P2..P7 alone) on an all-grassland map.
    pub(crate) fn test_state(width: i32, height: i32) -> GameState {
        let mut setup = GameSetup::standard(1, [Faction::Switzerland, Faction::Byzantium]);
        setup.turn_limit = Some(150);
        setup.width = width;
        setup.height = height;
        let map = Map::new(width, height, Terrain::Grassland);
        {
            let mut state = blank_game(&setup, map);
            // These pre-diplomacy tactical fixtures assume every foreign team is at War.
            for relation in &mut state.diplomacy.as_mut().unwrap().relations {
                relation.state = crate::diplomacy::RelationState::War;
            }
            state
        }
    }

    /// Insert a city directly and give it its radius-1 territory.
    pub(crate) fn add_city(state: &mut GameState, owner: PlayerId, pos: Hex, pop: i32) -> CityId {
        let id = state.alloc_city_id();
        let is_capital = state.capital_of(owner).is_none();
        state.cities.insert(
            id,
            City {
                last_attacked_turn: None,
                id,
                owner,
                founder: owner,
                name: format!("City {}", id.0),
                pos,
                population: pop,
                food_stored: 0,
                production_stored: 0,
                producing: None,
                production_banked: Vec::new(),
                bought: None,
                production_queue: Vec::new(),
                build_automation: None,
                last_auto_pick: None,
                auto_queued: None,
                production_queue_notified: Default::default(),
                wonder_lost: None,
                last_purchase_turn: None,
                buildings: BTreeSet::new(),
                hp: CITY_HP,
                grown_tiles: Vec::new(),
                tiles_claimed: 0,
                is_capital,
                original_capital: is_capital,
                greece_captive_paid: Default::default(),
                occupied_from: None,
                founded_turn: 1,
                resistance: 0,
            },
        );
        for h in pos.within(1) {
            if let Some(t) = state.map.get_mut(h) {
                t.owner = Some(owner);
                t.city = Some(id);
            }
        }
        id
    }

    fn unit(view: &PlayerView, id: UnitId) -> Option<&UnitView> {
        view.units.iter().find(|u| u.id == id)
    }

    fn city(view: &PlayerView, id: CityId) -> Option<&CityView> {
        view.cities.iter().find(|c| c.id == id)
    }

    #[test]
    fn fog_hides_enemy_units_outside_sight() {
        let mut s = test_state(24, 16);
        let mine = spawn_unit(&mut s, P0, UnitKind::Spearman, at(2, 2));
        let near = spawn_unit(&mut s, P2, UnitKind::Spearman, at(4, 2));
        let far = spawn_unit(&mut s, P2, UnitKind::Scout, at(18, 10));
        s.units.get_mut(&near).unwrap().goto = Some(at(0, 0));
        s.units.get_mut(&mine).unwrap().goto = Some(at(5, 5));
        visibility::update_explored(&mut s);

        let v = player_view(&s, P0);
        assert_eq!(unit(&v, mine).unwrap().goto, Some(at(5, 5)));
        let seen = unit(&v, near).expect("enemy in sight is shown");
        assert_eq!(seen.goto, None, "enemy destinations stay hidden");
        assert!(unit(&v, far).is_none(), "enemy outside sight is hidden");
        // Ids ascending.
        assert!(v.units.windows(2).all(|w| w[0].id < w[1].id));

        // The enemy sees our soldier (within its sight) but not the other way round for P3.
        let v2 = player_view(&s, P2);
        assert!(unit(&v2, mine).is_some());
        let v3 = player_view(&s, P3);
        assert!(v3.units.is_empty());
    }

    #[test]
    fn teammates_share_vision_and_see_each_others_units() {
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Spearman, at(2, 2));
        let b = spawn_unit(&mut s, P1, UnitKind::Scout, at(18, 10));
        s.units.get_mut(&b).unwrap().goto = Some(at(20, 10));
        let enemy = spawn_unit(&mut s, P2, UnitKind::Spearman, at(19, 10));
        visibility::update_explored(&mut s);

        let v = player_view(&s, P0);
        let mate = unit(&v, b).expect("teammate's unit is always shown");
        assert_eq!(mate.goto, None, "only the viewer's own gotos are shown");
        assert_eq!(mate.upgrade_cost, None);
        assert!(
            unit(&v, enemy).is_some(),
            "seen through the teammate's eyes"
        );
        assert!(v.tile(at(19, 10)).unwrap().visible);
        assert!(v.tile(at(19, 10)).unwrap().explored);

        let v1 = player_view(&s, P1);
        assert!(unit(&v1, a).is_some());
        assert!(v1.tile(at(2, 2)).unwrap().visible);
    }

    #[test]
    fn unexplored_tiles_have_no_terrain() {
        let mut s = test_state(24, 16);
        spawn_unit(&mut s, P0, UnitKind::Scout, at(2, 2));
        let far = at(20, 12);
        s.map.get_mut(far).unwrap().owner = Some(P2);
        visibility::update_explored(&mut s);

        let v = player_view(&s, P0);
        assert_eq!(v.tiles.len(), (24 * 16) as usize);
        let near = v.tile(at(3, 2)).unwrap();
        assert!(near.explored && near.visible);
        assert_eq!(near.terrain, Some(Terrain::Grassland));
        let t = v.tile(far).unwrap();
        assert!(!t.explored && !t.visible);
        assert_eq!(t.terrain, None);
        assert_eq!(t.owner, None, "owners of unexplored tiles are hidden");

        // Explored earlier but out of sight now: terrain and owner known, not visible.
        let scout = spawn_unit(&mut s, P0, UnitKind::Scout, far);
        visibility::update_explored(&mut s);
        s.units.remove(&scout);
        let v = player_view(&s, P0);
        let t = v.tile(far).unwrap();
        assert!(t.explored && !t.visible);
        assert_eq!(t.terrain, Some(Terrain::Grassland));
        assert_eq!(t.owner, Some(P2));
    }

    #[test]
    fn fog_reveals_a_city_founded_on_a_remembered_empty_tile() {
        let mut s = test_state(24, 16);
        let site = at(5, 5);
        let scout = spawn_unit(&mut s, P0, UnitKind::Scout, site);
        visibility::update_explored(&mut s);
        s.units.get_mut(&scout).unwrap().pos = at(20, 12);
        let settler = spawn_unit(&mut s, P2, UnitKind::Settler, site);
        let id = crate::rules::cities::found_city(&mut s, settler, &mut Vec::new()).unwrap();
        visibility::update_explored(&mut s);

        for viewer in [P0, P1] {
            let v = player_view(&s, viewer);
            assert!(v.tile(site).unwrap().explored);
            assert!(!v.tile(site).unwrap().visible);
            let founded = city(&v, id).expect("public city enters the AI's view");
            assert_eq!(
                (founded.owner, founded.name.as_str()),
                (P2, s.cities.get(&id).unwrap().name.as_str())
            );
            assert!(!founded.visible);
            assert_eq!((founded.population, founded.hp), (1, CITY_HP));
            assert_eq!(v.tile(site).unwrap().owner, Some(P2));
        }
        s.units.get_mut(&scout).unwrap().pos = site;
        visibility::update_explored(&mut s);
        assert_eq!(city(&player_view(&s, P0), id).unwrap().owner, P2);
    }

    #[test]
    fn fog_does_not_reveal_a_city_on_unexplored_land_or_create_contact() {
        let mut s = test_state(24, 16);
        let site = at(20, 12);
        let id = add_city(&mut s, P2, site, 3);
        let candidate = at(17, 12);
        let water = at(18, 12);
        s.map.get_mut(water).unwrap().terrain = Terrain::Ocean;
        let settler = spawn_unit(&mut s, P0, UnitKind::Settler, at(2, 2));
        let candidate_index = s.map.index(candidate).unwrap();
        s.players[0].explored.set(candidate_index);
        let water_index = s.map.index(water).unwrap();
        s.players[0].explored.set(water_index);
        visibility::update_explored(&mut s);
        let v = player_view(&s, P0);
        assert!(!v.tile(site).unwrap().explored);
        assert_eq!(v.tile(site).unwrap().owner, None);
        assert!(city(&v, id).is_none());
        assert!(v.tile(candidate).unwrap().explored);
        assert!(v.city_gap_blocked.contains(&candidate));
        assert!(
            !v.city_gap_blocked.contains(&water),
            "non-foundable terrain gets no hidden-city signal"
        );
        assert!(
            !v.city_gap_blocked.contains(&site),
            "unexplored centers stay out of the mask"
        );
        assert_eq!(
            crate::settlement::site_blocker(&v, candidate),
            Some(crate::settlement::SiteBlocker::CityTooClose)
        );
        assert!(!crate::settlement::known_legal_site(&v, candidate));
        assert!(
            crate::settlement::settlement_scores(&v, unit(&v, settler).unwrap(), &[])
                .iter()
                .all(|score| score.pos != candidate)
        );
        assert!(
            v.city_gap_blocked
                .iter()
                .all(|&pos| v.tile(pos).unwrap().explored)
        );
        let ai = player_view_detail(&s, P0, &public_view(&s), ViewDetail::Decisions);
        assert_eq!(
            ai.city_gap_blocked, v.city_gap_blocked,
            "AI and player get the same spacing flag"
        );
        let json = serde_json::to_string(&v).unwrap();
        assert!(!json.contains(&format!("City {}", id.0)));
        assert!(!visibility::contact_pairs(&s).contains(&(s.team_of(P0), s.team_of(P2))));

        // India can found three hexes away; the mask uses the founder's gap, not the rival's.
        s.players[0].faction = Faction::India;
        let india = player_view(&s, P0);
        assert!(!india.city_gap_blocked.contains(&candidate));
        assert!(crate::settlement::known_legal_site(&india, candidate));
    }

    #[test]
    fn fog_shows_current_city_owner_and_name_but_remembers_private_facts() {
        let mut s = test_state(24, 16);
        // Test a captor that keeps cities, independently of the rotating AI roster.
        s.player_mut(P3).faction = Faction::Byzantium;
        let site = at(5, 5);
        let id = add_city(&mut s, P2, site, 3);
        s.map.get_mut(site).unwrap().ruin = true;
        let scout = spawn_unit(&mut s, P0, UnitKind::Scout, at(4, 5));
        visibility::update_explored(&mut s);
        let remembered = city(&player_view(&s, P0), id).unwrap().clone();
        s.units.get_mut(&scout).unwrap().pos = at(20, 12);
        crate::rules::battle::capture_city(&mut s, id, P3, &mut Vec::new());
        let hidden_guard = spawn_unit(&mut s, P3, UnitKind::Spearman, site);
        let c = s.cities.get_mut(&id).unwrap();
        c.name = "Changed unseen".into();
        c.hp = 17;
        c.buildings.insert(BuildingKind::Walls);
        s.map.get_mut(site).unwrap().ruin = false;
        visibility::update_explored(&mut s);
        let memory = s.players[0].tile_memory[s.map.index(site).unwrap()]
            .as_ref()
            .unwrap();
        assert_eq!(memory.owner, Some(P3));
        assert_eq!(
            memory.city.as_ref().map(|c| (c.owner, c.name.as_str())),
            Some((P3, "Changed unseen"))
        );
        for viewer in [P0, P1] {
            let v = player_view(&s, viewer);
            let captured = city(&v, id).unwrap();
            assert_eq!(
                (captured.owner, captured.name.as_str()),
                (P3, "Changed unseen")
            );
            assert!(!captured.visible);
            assert_eq!(
                (captured.population, captured.hp, captured.walls),
                (remembered.population, remembered.hp, remembered.walls)
            );
            assert_eq!(v.tile(site).unwrap().owner, Some(P3));
            assert!(v.tile(site).unwrap().ruin);
            assert!(unit(&v, hidden_guard).is_none());
        }
        assert!(!visibility::contact_pairs(&s).contains(&(s.team_of(P0), s.team_of(P3))));
        let restored: GameState = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(restored, s);
        assert_eq!(restored.hash(), s.hash());
        assert_eq!(city(&player_view(&restored, P0), id).unwrap().owner, P3);
        s.units.get_mut(&scout).unwrap().pos = at(4, 5);
        // Visible data is live even before the next memory update.
        let v = player_view(&s, P0);
        let live = city(&v, id).unwrap();
        assert_eq!((live.owner, live.population, live.hp), (P3, 1, 17));
        assert!(live.walls && !live.is_capital);
        assert_eq!(live.name, "Changed unseen");
        assert!(!v.tile(site).unwrap().ruin);
        visibility::update_explored(&mut s);
        s.units.get_mut(&scout).unwrap().pos = at(20, 12);
        let latest = city(&player_view(&s, P0), id).unwrap().clone();
        assert!(!latest.visible);
        assert_eq!((latest.owner, latest.hp), (P3, 17));
    }

    #[test]
    fn fog_removes_a_razed_city_immediately() {
        let mut s = test_state(24, 16);
        let site = at(5, 5);
        let id = add_city(&mut s, P2, site, 3);
        let scout = spawn_unit(&mut s, P0, UnitKind::Scout, at(4, 5));
        visibility::update_explored(&mut s);
        s.units.get_mut(&scout).unwrap().pos = at(20, 12);
        s.cities.remove(&id);
        for tile in &mut s.map.tiles {
            if tile.city == Some(id) {
                tile.city = None;
                tile.owner = None;
            }
        }
        visibility::update_explored(&mut s);
        assert!(
            s.players[0].tile_memory[s.map.index(site).unwrap()]
                .as_ref()
                .unwrap()
                .city
                .is_none()
        );
        let v = player_view(&s, P0);
        assert!(city(&v, id).is_none(), "a razed city disappears under fog");
        assert_eq!(v.tile(site).unwrap().owner, None);
        s.units.get_mut(&scout).unwrap().pos = at(4, 5);
        visibility::update_explored(&mut s);
        s.units.get_mut(&scout).unwrap().pos = at(20, 12);
        assert!(city(&player_view(&s, P0), id).is_none());
        assert_eq!(player_view(&s, P0).tile(site).unwrap().owner, None);
    }

    #[test]
    fn own_cities_have_detail_enemies_do_not() {
        let mut s = test_state(24, 16);
        let mine = add_city(&mut s, P0, at(3, 3), 2);
        let mate = add_city(&mut s, P1, at(9, 3), 1);
        let near_enemy = add_city(&mut s, P2, at(5, 5), 3);
        let far_enemy = add_city(&mut s, P3, at(20, 13), 1);
        visibility::update_explored(&mut s);

        let v = player_view(&s, P0);
        let own = city(&v, mine).unwrap();
        let d = own.detail.as_ref().expect("own city has detail");
        assert_eq!(d.territory.len(), 7);
        assert!(d.territory.contains(&at(3, 3)));
        assert!(own.visible && own.is_capital);

        let m = city(&v, mate).expect("teammate city shown");
        assert!(m.detail.is_none());

        let e = city(&v, near_enemy).expect("enemy city on explored tile shown");
        assert!(e.detail.is_none());
        assert!(e.visible);
        assert_eq!(e.strength, 10 + 2 * 3);
        assert!(
            city(&v, far_enemy).is_none(),
            "unexplored enemy city hidden"
        );

        // Explored but out of sight: listed as not visible.
        let scout = spawn_unit(&mut s, P0, UnitKind::Scout, at(20, 13));
        visibility::update_explored(&mut s);
        s.units.remove(&scout);
        let v = player_view(&s, P0);
        assert!(!city(&v, far_enemy).unwrap().visible);
        assert!(v.cities.windows(2).all(|w| w[0].id < w[1].id));
        assert_eq!(v.players[0].cities, 1);
        assert_eq!(v.players[3].cities, 1);
    }

    #[test]
    fn completed_great_works_are_public_only_in_explored_cities() {
        let mut s = test_state(24, 16);
        let site = at(5, 5);
        let rival = add_city(&mut s, P2, site, 3);
        let hidden = add_city(&mut s, P3, at(20, 12), 3);
        for id in [rival, hidden] {
            let buildings = &mut s.cities.get_mut(&id).unwrap().buildings;
            buildings.insert(BuildingKind::GreatLibrary);
            buildings.insert(BuildingKind::Granary);
        }
        let scout = spawn_unit(&mut s, P0, UnitKind::Scout, at(4, 5));
        visibility::update_explored(&mut s);

        let v = player_view(&s, P0);
        let seen = city(&v, rival).unwrap();
        assert!(seen.visible);
        assert_eq!(seen.great_works, vec![BuildingKind::GreatLibrary]);
        assert!(seen.detail.is_none(), "rival buildings stay private");
        assert!(city(&v, hidden).is_none(), "unexplored city stays hidden");

        s.units.get_mut(&scout).unwrap().pos = at(20, 12);
        s.cities
            .get_mut(&rival)
            .unwrap()
            .buildings
            .insert(BuildingKind::GrandBazaar);
        visibility::update_explored(&mut s);
        let v = player_view(&s, P0);
        let fogged = city(&v, rival).unwrap();
        assert!(!fogged.visible);
        assert_eq!(
            fogged.great_works,
            vec![BuildingKind::GreatLibrary, BuildingKind::GrandBazaar]
        );
        assert!(fogged.detail.is_none());
    }

    #[test]
    fn air_sightings_are_seen_only_with_their_city_and_expose_no_private_details() {
        let mut s = test_state(24, 16);
        add_city(&mut s, P0, at(3, 3), 2);
        let near = add_city(&mut s, P2, at(5, 5), 2);
        let far = add_city(&mut s, P3, at(20, 12), 2);
        s.aircraft.push(crate::state::Aircraft {
            id: UnitId(90),
            kind: UnitKind::Fighter,
            city: near,
            hp: 41,
            xp: 0,
        });
        s.aircraft.push(crate::state::Aircraft {
            id: UnitId(91),
            kind: UnitKind::Bomber,
            city: far,
            hp: 85,
            xp: 0,
        });
        visibility::update_explored(&mut s);
        let seen = player_view(&s, P0);
        assert_eq!(
            seen.air_sightings,
            vec![AirSighting {
                owner: P2,
                kind: UnitKind::Fighter,
                city: near
            }]
        );
        assert!(
            seen.air_contacts.iter().any(|a| a.owner == P2
                && a.kind == Some(UnitKind::Fighter)
                && a.city == Some(near))
        );
        assert!(
            seen.aircraft.is_empty(),
            "rival health and supply remain private"
        );
        let scout = spawn_unit(&mut s, P0, UnitKind::Scout, at(20, 12));
        visibility::update_explored(&mut s);
        s.units.remove(&scout);
        let fogged = player_view(&s, P0);
        assert!(fogged.air_sightings.iter().all(|a| a.city != far));
        assert!(
            fogged
                .air_contacts
                .iter()
                .any(|a| a.owner == P3 && a.city == Some(far)),
            "a past sighting remains an observation, not a current hangar read"
        );
    }

    #[test]
    fn queued_wonder_tile_and_stage_are_public_even_in_fog() {
        let mut s = test_state(24, 16);
        let rival = add_city(&mut s, P2, at(5, 5), 3);
        let site = at(6, 5);
        let c = s.cities.get_mut(&rival).unwrap();
        c.producing = Some(ProductionItem::Wonder {
            building: BuildingKind::GreatLibrary,
            tile: site,
        });
        c.production_stored = 240;
        let scout = spawn_unit(&mut s, P0, UnitKind::Scout, at(4, 5));
        visibility::update_explored(&mut s);
        let first = player_view(&s, P0);
        let progress = city(&first, rival).unwrap().construction.unwrap();
        assert_eq!(
            (progress.building, progress.tile),
            (BuildingKind::GreatLibrary, site)
        );
        assert!(progress.progress > 0);
        s.units.get_mut(&scout).unwrap().pos = at(20, 12);
        visibility::update_explored(&mut s);
        let fogged = city(&player_view(&s, P0), rival).unwrap().clone();
        assert!(!fogged.visible);
        assert!(fogged.detail.is_none());
        assert_eq!(fogged.construction, Some(progress));
    }

    #[test]
    fn rival_walls_under_construction_are_public_even_in_fog() {
        let mut s = test_state(24, 16);
        let rival = add_city(&mut s, P2, at(5, 5), 3);
        let scout = spawn_unit(&mut s, P0, UnitKind::Scout, at(4, 5));
        visibility::update_explored(&mut s);
        assert!(!city(&player_view(&s, P0), rival).unwrap().walls_building);
        s.cities.get_mut(&rival).unwrap().producing = Some(ProductionItem::Building {
            building: BuildingKind::Walls,
        });
        assert!(city(&player_view(&s, P0), rival).unwrap().walls_building);
        s.units.get_mut(&scout).unwrap().pos = at(20, 12);
        visibility::update_explored(&mut s);
        let fogged = city(&player_view(&s, P0), rival).unwrap().clone();
        assert!(!fogged.visible);
        assert!(fogged.walls_building);
    }

    #[test]
    fn production_option_turns_include_only_unassigned_or_own_banked_production() {
        let spearman = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        let granary = ProductionItem::Building {
            building: BuildingKind::Granary,
        };
        for producing in [None, Some(granary)] {
            let mut s = test_state(24, 16);
            let id = add_city(&mut s, P0, at(5, 5), 1);
            let c = s.cities.get_mut(&id).unwrap();
            c.production_stored = 30;
            c.producing = producing;
            if producing.is_some() {
                // The Monument owns its 30; give the Spearman its own banked 30 instead.
                c.production_stored = 0;
                c.bank(spearman, 30);
            }
            let v = player_view(&s, P0);
            let detail = city(&v, id).unwrap().detail.as_ref().unwrap();
            let option = detail.options.iter().find(|o| o.item == spearman).unwrap();
            // Spearman 80, 30 stored, 5 a turn.
            assert_eq!(option.turns, 10);
            assert_eq!(option.production_stored, 30);
            assert_eq!(
                option.production_banked,
                if producing.is_some() { 30 } else { 0 }
            );
            crate::rules::cities::apply_city_orders(
                &mut s,
                P0,
                &[crate::Order::SetProduction {
                    city: id,
                    item: spearman,
                }],
                &mut Vec::new(),
            );
            for turn in 1..=10 {
                let mut events = Vec::new();
                crate::rules::cities::process_cities(&mut s, &mut events);
                assert_eq!(
                    events
                        .iter()
                        .any(|e| matches!(e, crate::Event::ProductionCompleted { .. })),
                    turn == 10
                );
            }
        }
    }

    #[test]
    fn unavailable_buildings_expose_only_a_missing_in_city_predecessor() {
        let mut s = test_state(24, 16);
        let id = add_city(&mut s, P0, at(5, 5), 5);
        s.players[0].techs.insert(TechId::Chivalry);
        let castle = ProductionItem::Building {
            building: BuildingKind::Castle,
        };
        let view = player_view(&s, P0);
        let unavailable = city(&view, id)
            .unwrap()
            .detail
            .as_ref()
            .unwrap()
            .unavailable
            .iter()
            .find(|entry| entry.item == castle)
            .unwrap();
        assert_eq!(unavailable.prerequisite_building, Some(BuildingKind::Walls));

        s.cities
            .get_mut(&id)
            .unwrap()
            .buildings
            .insert(BuildingKind::Walls);
        let view = player_view(&s, P0);
        let detail = city(&view, id).unwrap().detail.as_ref().unwrap();
        assert!(detail.unavailable.iter().all(|entry| entry.item != castle));
        assert!(detail.options.iter().any(|entry| entry.item == castle));

        let mut unresearched = test_state(24, 16);
        let id = add_city(&mut unresearched, P0, at(5, 5), 5);
        let view = player_view(&unresearched, P0);
        assert!(
            city(&view, id)
                .unwrap()
                .detail
                .as_ref()
                .unwrap()
                .unavailable
                .iter()
                .all(|entry| entry.item != castle)
        );
    }

    #[test]
    fn production_option_quotes_match_switch_validation_and_actual_spending() {
        let spearman = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        let granary = ProductionItem::Building {
            building: BuildingKind::Granary,
        };
        for faction in [Faction::India, Faction::Switzerland] {
            for producing in [None, Some(granary)] {
                for stored in [30, 1_000] {
                    let mut s = test_state(24, 16);
                    s.player_mut(P0).faction = faction;
                    let id = add_city(&mut s, P0, at(5, 5), 1);
                    let c = s.cities.get_mut(&id).unwrap();
                    c.production_stored = stored;
                    c.producing = producing;
                    let view = player_view(&s, P0);
                    let option = city(&view, id)
                        .unwrap()
                        .detail
                        .as_ref()
                        .unwrap()
                        .options
                        .iter()
                        .find(|o| o.item == spearman)
                        .unwrap();
                    // Unassigned Production joins the Spearman; a Monument's stays its own.
                    assert_eq!(
                        option.production_stored,
                        if producing.is_none() { stored } else { 0 }
                    );
                    let quote = option.buy_cost.unwrap();
                    if stored == 30 {
                        assert_eq!(
                            quote,
                            if faction == Faction::Switzerland {
                                140
                            } else {
                                160
                            }
                        );
                    }
                    if stored == 1_000 {
                        assert!(quote > 0, "stored production never discounts purchases");
                    }
                    let orders = [
                        crate::Order::SetProduction {
                            city: id,
                            item: spearman,
                        },
                        crate::Order::BuyItem {
                            city: id,
                            item: spearman,
                        },
                    ];
                    s.player_mut(P0).gold = quote;
                    assert!(crate::rules::cities::validate_city_orders(&s, P0, &orders).is_empty());
                    if quote > 0 {
                        s.player_mut(P0).gold = quote - 1;
                        assert_eq!(
                            crate::rules::cities::validate_city_orders(&s, P0, &orders)[0].index,
                            1
                        );
                        s.player_mut(P0).gold = quote;
                    }
                    let mut events = Vec::new();
                    crate::rules::cities::apply_city_orders(&mut s, P0, &orders, &mut events);
                    assert_eq!(s.player(P0).gold, 0);
                    assert!(events.contains(&crate::Event::ItemBought {
                        city: id,
                        owner: P0,
                        item: spearman,
                        gold: quote
                    }));
                }
            }
        }
    }

    #[test]
    fn separate_purchases_do_not_change_production_forecasts() {
        let mut s = test_state(24, 16);
        let id = add_city(&mut s, P0, at(5, 5), 1);
        let spearman = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        let granary = ProductionItem::Building {
            building: BuildingKind::Granary,
        };
        s.cities.get_mut(&id).unwrap().production_stored = 10;
        s.player_mut(P0).gold = 1_000;
        crate::rules::cities::apply_city_orders(
            &mut s,
            P0,
            &[
                crate::Order::SetProduction {
                    city: id,
                    item: spearman,
                },
                crate::Order::BuyItem {
                    city: id,
                    item: spearman,
                },
            ],
            &mut Vec::new(),
        );
        let v = player_view(&s, P0);
        let detail = city(&v, id).unwrap().detail.as_ref().unwrap();
        let bought = detail.options.iter().find(|o| o.item == spearman).unwrap();
        assert!(
            bought
                .buy_blocker
                .as_ref()
                .unwrap()
                .contains("already bought")
        );
        let switched = detail.options.iter().find(|o| o.item == granary).unwrap();
        // The 10 unassigned joined the Spearman when it started; a Monument starts from 0.
        assert_eq!(switched.production_stored, 0);
        let quote = switched.buy_cost;
        assert_eq!(quote, Some(switched.cost * 175 / 100));
        let mut projected = s.clone();
        crate::rules::cities::apply_city_orders(
            &mut projected,
            P0,
            &[crate::Order::SetProduction {
                city: id,
                item: granary,
            }],
            &mut Vec::new(),
        );
        assert_eq!(economy::buy_cost(&projected, &projected.cities[&id]), quote);
        assert_eq!(
            detail
                .options
                .iter()
                .find(|o| o.item == spearman)
                .unwrap()
                .turns,
            14
        );
        assert_eq!(
            detail
                .options
                .iter()
                .find(|o| o.item == granary)
                .unwrap()
                .turns,
            10
        );
    }

    #[test]
    fn bought_settlers_escalate_the_active_build_cost_and_resistance_still_blocks() {
        let mut s = test_state(24, 16);
        // Population 3: the bought Settler leaves 2, so the active Settler stays buildable.
        let id = add_city(&mut s, P0, at(5, 5), 3);
        let settler = ProductionItem::Unit {
            unit: UnitKind::Settler,
        };
        s.player_mut(P0).gold = 1_000;
        crate::rules::cities::apply_city_orders(
            &mut s,
            P0,
            &[
                crate::Order::SetProduction {
                    city: id,
                    item: settler,
                },
                crate::Order::BuyItem {
                    city: id,
                    item: settler,
                },
            ],
            &mut Vec::new(),
        );
        s.player_mut(P0).settlers_trained = 1;
        let v = player_view(&s, P0);
        let detail = city(&v, id).unwrap().detail.as_ref().unwrap();
        assert_eq!(detail.production_cost, Some(120));
        assert_ne!(detail.turns_to_complete, Some(1));
        let option = detail.options.iter().find(|o| o.item == settler).unwrap();
        assert_eq!(option.cost, 120);
        assert!(option.turns > 1);
        assert!(
            option
                .buy_blocker
                .as_ref()
                .unwrap()
                .contains("already bought")
        );
        s.cities.get_mut(&id).unwrap().resistance = 2;
        let v = player_view(&s, P0);
        let detail = city(&v, id).unwrap().detail.as_ref().unwrap();
        assert_eq!(detail.turns_to_complete, None);
        assert!(detail.options.iter().all(|o| o.turns == -1));
    }

    #[test]
    fn fully_funded_research_takes_one_turn_without_science_income() {
        let cost_of = |s: &GameState, tech| economy::tech_cost(s, P0, tech).total;
        let irrigation = cost_of(&test_state(24, 16), TechId::Irrigation);
        for stored in [irrigation, irrigation + 5] {
            let mut s = test_state(24, 16);
            let city = add_city(&mut s, P0, at(5, 5), 1);
            s.cities.get_mut(&city).unwrap().is_capital = false;
            s.player_mut(P0).science_stored = stored;
            let v = player_view(&s, P0);
            assert_eq!(v.economy.science_per_turn.total, 0);
            assert_eq!(
                v.research
                    .available
                    .iter()
                    .find(|o| o.tech == TechId::Irrigation)
                    .unwrap()
                    .turns,
                Some(1)
            );
            // Another tech is funded the same way if the bank covers it, else it never arrives.
            assert_eq!(
                v.research
                    .available
                    .iter()
                    .find(|o| o.tech == TechId::Masonry)
                    .unwrap()
                    .turns,
                (stored >= cost_of(&s, TechId::Masonry)).then_some(1)
            );
            crate::rules::cities::apply_city_orders(
                &mut s,
                P0,
                &[crate::Order::SetResearch {
                    tech: TechId::Irrigation,
                }],
                &mut Vec::new(),
            );
            assert_eq!(player_view(&s, P0).research.turns_left, Some(1));
            crate::rules::cities::process_cities(&mut s, &mut Vec::new());
            crate::rules::cities::process_research(&mut s, &mut Vec::new());
            assert!(s.player(P0).techs.contains(&TechId::Irrigation));
            assert_eq!(s.player(P0).science_stored, stored - irrigation);
        }
    }

    #[test]
    fn growth_and_production_turns() {
        let mut s = test_state(24, 16);
        let pos = at(5, 5);
        let id = add_city(&mut s, P0, pos, 2);
        // Two forest tiles (food 1, prod 2) outscore grassland under Balanced focus.
        let forests: Vec<Hex> = pos.ring(1).into_iter().take(2).collect();
        for h in &forests {
            s.map.get_mut(*h).unwrap().terrain = Terrain::Forest;
        }
        {
            let c = s.cities.get_mut(&id).unwrap();
            c.food_stored = 10;
            c.production_stored = 12;
            c.producing = Some(ProductionItem::Building {
                building: BuildingKind::Granary,
            });
            c.tiles_claimed = 2;
        }
        visibility::update_explored(&mut s);

        let v = player_view(&s, P0);
        let d = city(&v, id).unwrap().detail.clone().unwrap();
        // Ring 1 is all worked. Food: center 2 + grassland 2 + forests 1 + 1 + four grassland
        // 8 = 14; production: center 1 + forests 2 + 2 + capital 2 = 7.
        assert_eq!(d.yields.total.food, 14);
        assert_eq!(d.yields.total.production, 7);
        assert_eq!(d.yields.worked.len(), 6);
        // Threshold 15 + 8 = 23; surplus 14 - 4 = 10; (23 - 10) / 10 rounded up = 2.
        assert_eq!(d.growth_threshold, 46);
        assert_eq!(d.turns_to_grow, Some(4));
        // Granary 50, 12 stored, 7 per turn: 38 / 7 rounded up = 6.
        assert_eq!(d.production_cost, Some(50));
        assert_eq!(d.turns_to_complete, Some(6));
        // Switzerland buy a separate full-price Granary: 50 * 1.75 = 87 (rounded down).
        assert_eq!(d.buy_cost, Some(87));
        // The next growth claims a worked ring-2 grassland tile.
        let claim = d.next_claim.clone().unwrap();
        assert_eq!(claim.tile.distance(pos), 2);
        assert_eq!((claim.yields.food, claim.worked), (2, true));

        let turns_for = |item: ProductionItem| {
            d.options
                .iter()
                .find(|o| o.item == item)
                .map(|o| (o.cost, o.turns))
        };
        let granary = ProductionItem::Building {
            building: BuildingKind::Granary,
        };
        let spearman = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        let market = ProductionItem::Building {
            building: BuildingKind::Market,
        };
        assert_eq!(turns_for(granary), Some((50, 6)), "counts stored progress");
        assert_eq!(
            turns_for(spearman),
            Some((80, 12)),
            "the Granary's progress stays banked with it; a Spearman starts from its own"
        );
        assert_eq!(turns_for(market), None, "needs Exchange");
        assert!(
            turns_for(ProductionItem::Unit {
                unit: UnitKind::Pikeman
            })
            .is_none()
        );

        // Starving city: no growth ETA. Resistance: no output, so no production ETA.
        {
            let c = s.cities.get_mut(&id).unwrap();
            c.population = 5;
            c.resistance = 2;
        }
        let v = player_view(&s, P0);
        let d = city(&v, id).unwrap().detail.clone().unwrap();
        assert_eq!(d.turns_to_grow, None);
        assert_eq!(d.turns_to_complete, None);
        assert_eq!(d.resistance, 2);
        assert!(d.options.iter().all(|o| o.turns == -1));
    }

    #[test]
    fn economy_and_army_are_public() {
        let mut s = test_state(24, 16);
        add_city(&mut s, P0, at(3, 3), 3);
        add_city(&mut s, P2, at(15, 10), 5);
        let v = player_view(&s, P0);
        assert_eq!(v.players[2].economy, scoring::economy_value(&s, P2));
        assert_eq!(v.players[0].economy, scoring::economy_value(&s, P0));
        assert_eq!(v.players[2].army, scoring::army_strength(&s, P2));
        assert_eq!(v.players[2].vp, 0);
        assert!(v.players[2].vp_sources.is_empty());
    }

    #[test]
    fn standings_sum_teams() {
        let mut s = test_state(24, 16);
        s.players[1].faction_goal_done = true;
        // The Remnant's goal is held by whoever holds its Heir: here the Remnant itself.
        let heir = crate::game::spawn_unit(
            &mut s,
            P1,
            UnitKind::HeirToTheThrone,
            Hex::from_offset(3, 3),
        );
        s.units.get_mut(&heir).unwrap().heir_of = Some(P1);
        s.turn = 42;
        let v = player_view(&s, P0);
        assert_eq!(v.standings.len(), 7);
        let team0 = &v.standings[0];
        assert_eq!(team0.members, vec![P0, P1]);
        assert_eq!(team0.vp, 2); // Faction goal.
        assert_eq!(team0.target, 20);
        let p3_team = v.standings.iter().find(|t| t.members == vec![P3]).unwrap();
        assert_eq!(p3_team.vp, 0);
        assert_eq!(p3_team.target, 10);
        assert!(v.players[1].faction_goal_done);
        assert_eq!((v.turn, v.turn_limit), (42, Some(150)));
        assert_eq!((v.winner, v.finished), (None, false));
    }

    #[test]
    fn upgrade_cost_needs_territory_tech_and_gold() {
        let mut s = test_state(24, 16);
        add_city(&mut s, P0, at(3, 3), 1);
        let inside = spawn_unit(&mut s, P0, UnitKind::Phalangite, at(4, 3));
        let outside = spawn_unit(&mut s, P0, UnitKind::Phalangite, at(8, 8));
        let scout = spawn_unit(&mut s, P0, UnitKind::Scout, at(3, 4));
        s.players[0].gold = 200;
        let cost = |s: &GameState, id| unit(&player_view(s, P0), id).unwrap().upgrade_cost;

        assert_eq!(cost(&s, inside), None, "needs Feudalism");
        s.players[0].techs.insert(TechId::Feudalism);
        // Phalangite→Pikeman also uses 1 Iron supply: an Iron deposit inside the border.
        s.players[0]
            .techs
            .extend([TechId::BronzeWorking, TechId::IronWorking]);
        s.map.get_mut(at(3, 3).neighbors()[0]).unwrap().resource =
            Some(crate::resources::Resource::Iron);
        // 2 x (76 - 55) = 42, doubled for Standard costs.
        assert_eq!(cost(&s, inside), Some(84));
        assert_eq!(cost(&s, outside), None, "outside own territory");
        assert_eq!(cost(&s, scout), None, "nothing to upgrade to");
        s.players[0].gold = 83;
        assert_eq!(cost(&s, inside), None, "not enough gold");
    }

    #[test]
    fn economy_and_research() {
        let mut s = test_state(24, 16);
        add_city(&mut s, P0, at(3, 3), 2);
        for kind in [UnitKind::Spearman, UnitKind::Spearman, UnitKind::Archer] {
            spawn_unit(&mut s, P0, kind, at(3, 3));
        }
        s.players[0].gold = 33;
        s.players[0].science_stored = 5;
        s.players[0].researching = Some(TechId::Irrigation);
        s.players[2].techs.insert(TechId::Irrigation);
        let v = player_view(&s, P0);

        assert_eq!(v.economy.gold, 33);
        assert_eq!(v.economy.upkeep, 3, "3 Swiss military units, 2 free");
        let income = economy::player_income(&s, P0);
        assert_eq!(v.economy.gold_per_turn, income.gold);
        // Science: pop 2 / 2 + 2 from the capital = 3 per turn.
        assert_eq!(v.economy.science_per_turn.total, 3);

        let r = &v.research;
        assert_eq!(r.current, Some(TechId::Irrigation));
        // Base cost, -10% because one other empire knows it.
        let irrigation = data::tech_stats(TechId::Irrigation).cost * 90 / 100;
        assert_eq!(r.cost, Some(irrigation));
        assert_eq!(r.turns_left, Some((irrigation - 5 + 2) / 3));
        assert!(r.known.is_empty());
        // The strict gate: only the five Ancient techs, nothing from the Classical era.
        let avail: Vec<TechId> = r.available.iter().map(|t| t.tech).collect();
        assert_eq!(
            avail,
            vec![
                TechId::Irrigation,
                TechId::BronzeWorking,
                TechId::Masonry,
                TechId::Currency,
                TechId::Writing
            ]
        );
        let bronze = data::tech_stats(TechId::BronzeWorking).cost;
        assert_eq!(r.available[1].cost.total, bronze);
        assert_eq!(r.available[1].turns, Some((bronze - 5 + 2) / 3));
        assert!(r.available.iter().all(|t| t.researchable && t.era == 0));
        assert_eq!((r.era, r.era_name.as_str()), (0, "Ancient"));
        assert_eq!((r.era_total, r.era_known), (5, 0));
        let locked = r
            .catalog
            .iter()
            .find(|t| t.tech == TechId::Mathematics)
            .unwrap();
        assert!(!locked.researchable);
        assert_eq!(locked.era, 1);

        s.players[0].techs.insert(TechId::BronzeWorking);
        let v = player_view(&s, P0);
        assert!(
            !v.research
                .available
                .iter()
                .any(|t| t.tech == TechId::BronzeWorking),
            "a known tech is no longer offered"
        );
        let writing = v
            .research
            .available
            .iter()
            .find(|t| t.tech == TechId::Writing)
            .unwrap();
        assert!(writing.researchable, "no prerequisites inside an era");
        assert_eq!(v.research.known, vec![TechId::BronzeWorking]);
        assert_eq!((v.research.era_total, v.research.era_known), (5, 1));
    }

    #[test]
    fn view_is_deterministic() {
        let mut s = test_state(24, 16);
        add_city(&mut s, P0, at(3, 3), 2);
        add_city(&mut s, P2, at(7, 4), 2);
        spawn_unit(&mut s, P2, UnitKind::Spearman, at(6, 3));
        visibility::update_explored(&mut s);
        let a = serde_json::to_string(&player_view(&s, P0)).unwrap();
        let b = serde_json::to_string(&player_view(&s.clone(), P0)).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn performance_full_size_map() {
        let mut s = test_state(data::MAP_WIDTH, data::MAP_HEIGHT);
        // 30 cities spread over 8 empires, 80 units.
        let mut n = 0;
        for row in 0..5 {
            for col in 0..6 {
                let owner = PlayerId((n % 8) as u8);
                add_city(&mut s, owner, at(3 + col * 8, 3 + row * 6), 1 + n % 6);
                n += 1;
            }
        }
        for i in 0..80 {
            let owner = PlayerId((i % 8) as u8);
            let kind = UnitKind::ALL[(i % 5) as usize];
            spawn_unit(&mut s, owner, kind, at((i * 7) % 48, (i * 5) % 32));
        }
        visibility::update_explored(&mut s);
        let start = std::time::Instant::now();
        let runs = 20u32;
        let mut units = 0;
        for p in 0..runs {
            units += player_view(&s, PlayerId((p % 8) as u8)).units.len();
        }
        let per_view = start.elapsed() / runs;
        println!("player_view on 48x32, 30 cities, 80 units: {per_view:?} per view");
        assert!(units > 0);
    }

    #[test]
    fn every_empires_gold_is_public_to_every_viewer() {
        let mut s = test_state(12, 12);
        s.players[0].gold = 123;
        s.players[1].gold = 456;
        s.players[2].gold = 789;
        for viewer in [P0, P1, P2] {
            let v = player_view(&s, viewer);
            let gold = |p: PlayerId| v.players.iter().find(|x| x.id == p).unwrap().gold;
            assert_eq!(
                [gold(P0), gold(P1), gold(P2)],
                [123, 456, 789],
                "viewer {viewer:?}"
            );
        }
        s.players[2].alive = false;
        let v = player_view(&s, P0);
        assert_eq!(
            v.players.iter().find(|x| x.id == P2).unwrap().gold,
            0,
            "the dead hold nothing"
        );
    }
}

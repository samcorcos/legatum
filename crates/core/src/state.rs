use crate::bitset::BitSet;
use crate::data;
use crate::hex::Hex;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// How well an AI empire plays. Difficulty changes only how the AI decides, never its resources
/// or rules (pillar 6). Ignored for human players.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AiDifficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl AiDifficulty {
    pub fn is_normal(&self) -> bool {
        *self == AiDifficulty::Normal
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerSetup {
    pub name: String,
    pub faction: Faction,
    pub team: TeamId,
    pub is_human: bool,
    /// Omitted when Normal, so saves, tapes and state hashes from before difficulty existed are
    /// unchanged.
    #[serde(default, skip_serializing_if = "AiDifficulty::is_normal")]
    pub difficulty: AiDifficulty,
    /// Hard+Search: this Hard AI decides its plans by rollouts on the engine (`legatum_ai::search`).
    /// An AI choice, not a rule: it changes only which orders the seat submits, never a resource
    /// or a price. Ignored unless the seat is an AI at Hard. Omitted when false, so saves, tapes
    /// and hashes from before it existed are unchanged.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub search: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSetup {
    #[serde(
        default,
        skip_serializing_if = "crate::experiments::RuleExperiments::is_default"
    )]
    pub rule_experiments: crate::experiments::RuleExperiments,
    pub seed: u64,
    pub width: i32,
    pub height: i32,
    pub players: Vec<PlayerSetup>,
    /// None means play until victory. Serde also accepts legacy numeric limits.
    #[serde(default)]
    pub turn_limit: Option<u32>,
    /// Start in the legacy all-war ruleset (no diplomacy state). Only the Proxy benchmark
    /// uses it, so its numbers stay comparable with its all-war calibration. Omitted from
    /// saves when false, so every existing setup, tape and hash is unchanged.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub legacy_all_war: bool,
    /// Civ 6 style barbarian camps (`rules::barbarians`). On for new games; saves and setups
    /// from before barbarians existed have no field and load with them off, and an Off setup
    /// serializes exactly as before, so old tapes and hashes are unchanged.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub barbarians: bool,
}

impl GameSetup {
    /// Keep established seeded setups and their order pins on the original nine-faction draw.
    /// Egypt and Rome are available through explicit seats, including the app's faction picker.
    fn seeded_faction_pool() -> Vec<Faction> {
        Faction::ALL[..9].to_vec()
    }

    /// The standard game: two humans on team 0, six AIs on their own teams.
    pub fn standard(seed: u64, human_factions: [Faction; 2]) -> Self {
        let mut players = vec![
            PlayerSetup {
                name: "Player 1".into(),
                faction: human_factions[0],
                team: TeamId(0),
                is_human: true,
                difficulty: AiDifficulty::Normal,
                search: false,
            },
            PlayerSetup {
                name: "Player 2".into(),
                faction: human_factions[1],
                team: TeamId(0),
                is_human: true,
                difficulty: AiDifficulty::Normal,
                search: false,
            },
        ];
        // Keep existing seeded rosters stable. Egypt and Rome enter AI seats when explicitly
        // selected or assigned by the app lobby; old simulation pins keep their roster.
        let pool: Vec<_> = Faction::ALL[..9]
            .iter()
            .copied()
            .filter(|f| !human_factions.contains(f))
            .collect();
        for i in 0..6u8 {
            players.push(PlayerSetup {
                name: format!("AI {}", i + 1),
                faction: pool[((seed % pool.len() as u64) as usize + i as usize) % pool.len()],
                team: TeamId(i + 1),
                is_human: false,
                difficulty: AiDifficulty::Normal,
                search: false,
            });
        }
        Self {
            seed,
            width: data::MAP_WIDTH,
            height: data::MAP_HEIGHT,
            players,
            turn_limit: None,
            legacy_all_war: false,
            barbarians: true,
            rule_experiments: Default::default(),
        }
    }

    /// All-AI game with every empire on its own team, for simulations.
    pub fn all_ai(seed: u64) -> Self {
        let pool = Self::seeded_faction_pool();
        let players = (0..8u8)
            .map(|i| PlayerSetup {
                name: format!("AI {}", i + 1),
                faction: pool[((seed % pool.len() as u64) as usize + i as usize) % pool.len()],
                team: TeamId(i),
                is_human: false,
                difficulty: AiDifficulty::Normal,
                search: false,
            })
            .collect();
        Self {
            seed,
            width: data::MAP_WIDTH,
            height: data::MAP_HEIGHT,
            players,
            turn_limit: None,
            legacy_all_war: false,
            barbarians: true,
            rule_experiments: Default::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: Terrain,
    /// An unexplored ancient ruin. Cleared when a unit first enters.
    pub ruin: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wonder: Option<crate::wonders::Wonder>,
    /// A built Great Work or capturable goal building occupying this tile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub landmark: Option<crate::landmarks::Landmark>,
    pub owner: Option<PlayerId>,
    /// The city whose territory this tile belongs to.
    pub city: Option<CityId>,
    /// A map resource (G11). Omitted when absent, so resource-free maps keep their hashes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<crate::resources::Resource>,
    /// The resource on this tile has been improved by its owning city (`crate::improvements`).
    /// Cleared whenever the tile changes hands. Omitted when false, so old hashes hold.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub improved: bool,
    /// A road here (`crate::roads`): bit d (0..6) links to the neighbour in direction d,
    /// `roads::NODE` marks the tile as road even with no link. 0 = no road. Roads never
    /// leave a tile, whoever owns it. Omitted at 0, so road-free maps keep their hashes.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub road: u8,
}

impl Tile {
    pub fn new(terrain: Terrain) -> Self {
        Self {
            terrain,
            ruin: false,
            wonder: None,
            landmark: None,
            owner: None,
            city: None,
            resource: None,
            improved: false,
            road: 0,
        }
    }

    pub fn is_road(&self) -> bool {
        self.road != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    /// Row-major, odd-r offset: index = row * width + col.
    pub tiles: Vec<Tile>,
    /// Where the Yam Relay Station's road network is rooted: the first Station's city tile
    /// (`crate::roads`). Omitted when `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub road_hub: Option<Hex>,
    /// Whether any tile carries a road (Station roads or a city's Roadworks). Omitted when false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub roaded: bool,
    /// Line-of-sight footprints already worked out on this map (`visibility::SightMemo`),
    /// shared by clones. Never serialized or compared; every entry is checked against the
    /// terrain it was computed from before it is reused.
    #[serde(skip)]
    pub(crate) sight: crate::visibility::SightMemo,
}

impl Map {
    pub fn new(width: i32, height: i32, fill: Terrain) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::new(fill); (width * height) as usize],
            road_hub: None,
            roaded: false,
            sight: Default::default(),
        }
    }

    pub fn index(&self, h: Hex) -> Option<usize> {
        let (c, r) = h.to_offset();
        if c < 0 || r < 0 || c >= self.width || r >= self.height {
            None
        } else {
            Some((r * self.width + c) as usize)
        }
    }

    pub fn contains(&self, h: Hex) -> bool {
        self.index(h).is_some()
    }

    pub fn get(&self, h: Hex) -> Option<&Tile> {
        self.index(h).map(|i| &self.tiles[i])
    }

    pub fn get_mut(&mut self, h: Hex) -> Option<&mut Tile> {
        self.index(h).map(|i| &mut self.tiles[i])
    }

    pub fn hex_at(&self, index: usize) -> Hex {
        let i = index as i32;
        Hex::from_offset(i % self.width, i / self.width)
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    /// Every hex in index order.
    pub fn hexes(&self) -> impl Iterator<Item = Hex> + '_ {
        (0..self.tiles.len()).map(|i| self.hex_at(i))
    }

    /// Movement cost for a land unit entering `h`, or `None` if impassable or off-map. A road
    /// tile costs 1 whatever its terrain (`crate::roads`).
    pub fn move_cost(&self, h: Hex) -> Option<i32> {
        self.get(h).and_then(|t| {
            t.terrain
                .move_cost()
                .map(|c| if t.is_road() { 1 } else { c })
        })
    }

    /// Whether any tile carries a road.
    pub fn has_roads(&self) -> bool {
        self.roaded
    }

    pub fn is_road(&self, h: Hex) -> bool {
        self.get(h).is_some_and(Tile::is_road)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub id: UnitId,
    pub owner: PlayerId,
    pub kind: UnitKind,
    pub pos: Hex,
    pub hp: i32,
    pub moves_left: i32,
    /// Persistent destination; the core pathfinds each turn.
    pub goto: Option<Hex>,
    /// Remaining confirmed route. Empty for ordinary, automatically pathed gotos.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<Hex>,
    /// A constrained route stops when blocked instead of automatically rerouting.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub path_stop_on_block: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automation: Option<Automation>,
    /// Only an explicit order suspends the order cycle across turns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standing_order: Option<StandingOrder>,
    /// Stopped automation requests a new decision even while fortified, until ordered again.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub needs_orders: bool,
    /// Consecutive turns this unit's move made no progress because its next step was blocked.
    /// At `MOVE_BLOCKED_STOP_TURNS` the order stops and the unit asks for new orders.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub blocked_turns: u8,
    /// The unit an explicit melee attack order targets. The attack follows the unit, not its
    /// tile: while set, movement re-aims `goto` at the target's position (see movement.rs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_target: Option<UnitId>,
    /// Bought Siege (`Order::UpgradeSiege`, `rules::siege`): this unit's attacks on a city ignore
    /// the Walls its owner's siege technology reaches. Permanent, kept through unit upgrades
    /// until one into an Industrial or later kind, which has Siege built in and drops this
    /// (`data::built_in_siege`; it is always false on those). Public (the siege gear is drawn
    /// beside it). Omitted when false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub siege: bool,
    /// Combat experience, cumulative; the promotion level derives from it
    /// (`rules::promotion::level`). Kept through upgrades. Omitted at 0.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub xp: u32,
    /// Fortification defense bonus (0..=FORTIFY_MAX), +3 per full idle turn.
    pub fortify: i32,
    /// Moved or fought this turn (no healing).
    pub acted: bool,
    /// Creation turn is an owner-known fact, persisted so AI commitment deadlines
    /// survive save/load. Starting units use turn one.
    #[serde(
        default = "unit_creation_turn",
        skip_serializing_if = "initial_unit_creation"
    )]
    pub created_turn: u32,
    /// Teams that a non-Celts unit attacked. Paired with `attack_revealed_turn`, this gives the
    /// victim one planning turn of sight even when the attacker is outside tile visibility.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub attack_revealed_to: BTreeSet<TeamId>,
    /// Turn of the latest ordinary attack reveal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_revealed_turn: Option<u32>,
    /// An Heir only: the Byzantium empire whose goal he carries. A captured Heir keeps it, so
    /// whoever holds him scores that empire's goal (`goals::heir_holder`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heir_of: Option<PlayerId>,
    /// A Mercenary Company only (Byzantium, `company`): the last turn of its contract. The unit
    /// goes home at the end of that turn's resolution. None for every ordinary unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_until: Option<u32>,
    /// A Mercenary Company only: the faction that sold it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Faction>,
}

fn unit_creation_turn() -> u32 {
    1
}

fn initial_unit_creation(turn: &u32) -> bool {
    *turn == 1
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionPurchase {
    pub item: ProductionItem,
    /// Full production cost quoted when the item was bought.
    pub cost: i32,
    /// The part of production_stored bought with gold; forfeited when switching items.
    pub progress: i32,
}

/// Progress a city has put into an item it switched away from (Civ 6 style). It waits for
/// the next time the city builds that item; it never transfers to a different item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BankedProduction {
    pub item: ProductionItem,
    pub progress: i32,
}

/// A rival completed the world wonder this city was actively building
/// (`rules/cities.rs::resolve_wonder_race`): `production` banked on it, `gold` refunded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonderLoss {
    pub turn: u32,
    pub winner: PlayerId,
    pub building: BuildingKind,
    pub production: i32,
    pub gold: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct City {
    /// The last turn the city took damage in battle; it doesn't heal at the end of that turn.
    /// Absent until the city is first damaged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_attacked_turn: Option<u32>,
    pub id: CityId,
    pub owner: PlayerId,
    pub founder: PlayerId,
    pub name: String,
    pub pos: Hex,
    pub population: i32,
    pub food_stored: i32,
    /// Progress of `producing`. With no unit or building active (idle, waiting, or a
    /// Germany project) it is unassigned Production (overflow and idle banking), which
    /// joins whatever the city starts next.
    pub production_stored: i32,
    pub producing: Option<ProductionItem>,
    /// Per-item progress banked when the city switched away, sorted by item. Omitted when
    /// empty so states without banked progress keep their serialization and hashes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub production_banked: Vec<BankedProduction>,
    /// Future or temporarily blocked builds.
    #[serde(default)]
    pub production_queue: Vec<ProductionItem>,
    /// None in cities controlled by hand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_automation: Option<CityBuildFocus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_auto_pick: Option<ProductionItem>,
    /// Future item placed by automation, removed if the setting is turned off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_queued: Option<ProductionItem>,
    /// Retained items whose current blockage has already been reported. Survives save/load.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub production_queue_notified: BTreeSet<ProductionItem>,
    /// The wonder race this city lost while building the wonder. The city stops and asks its
    /// owner to choose; its queue waits until its next production phase, which clears this, as
    /// does any production order or a capture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wonder_lost: Option<WonderLoss>,
    #[serde(default)]
    pub last_purchase_turn: Option<u32>,
    #[serde(default)]
    pub bought: Option<ProductionPurchase>,
    pub buildings: BTreeSet<BuildingKind>,
    pub hp: i32,
    /// Tiles gained after founding and still held through population growth.
    pub tiles_claimed: i32,
    /// Tiles claimed by population growth, oldest first (population is territory: one per
    /// population above 1). Losing population releases one of them; the founding ring is never
    /// released.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grown_tiles: Vec<Hex>,
    pub is_capital: bool,
    /// Remains true after conquest, unlike `is_capital`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub original_capital: bool,
    /// Empires already paid for capturing this city while Athens owned it.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub greece_captive_paid: BTreeSet<PlayerId>,
    /// Occupation lock (`rules::occupation`): the civ the last captor took this city from. Set at
    /// capture only with the experiment on; omitted when absent, so saves and hashes are unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occupied_from: Option<PlayerId>,
    pub founded_turn: u32,
    /// Turns of post-capture resistance left (no output while > 0).
    pub resistance: i32,
}

impl City {
    /// Whether the active build owns `production_stored`. Projects never hold progress, so
    /// with a project (or nothing) active the stored amount is unassigned.
    pub fn holds_progress(&self) -> bool {
        matches!(
            self.producing,
            Some(
                ProductionItem::Unit { .. }
                    | ProductionItem::Building { .. }
                    | ProductionItem::Wonder { .. }
                    | ProductionItem::Improvement { .. }
            )
        )
    }

    /// Progress banked for `item` from an earlier switch (0 when none).
    pub fn banked(&self, item: ProductionItem) -> i32 {
        self.production_banked
            .iter()
            .find(|b| b.item == item)
            .map_or(0, |b| b.progress)
    }

    /// Add progress to `item`'s bank. Projects never bank.
    pub fn bank(&mut self, item: ProductionItem, progress: i32) {
        if progress <= 0 || matches!(item, ProductionItem::Project { .. }) {
            return;
        }
        match self
            .production_banked
            .binary_search_by(|b| b.item.cmp(&item))
        {
            Ok(i) => {
                self.production_banked[i].progress =
                    crate::validation::add_stored(self.production_banked[i].progress, progress)
            }
            Err(i) => self.production_banked.insert(
                i,
                BankedProduction {
                    item,
                    progress: progress.min(crate::validation::MAX_STORED),
                },
            ),
        }
    }

    /// Remove and return `item`'s banked progress.
    pub fn take_banked(&mut self, item: ProductionItem) -> i32 {
        match self
            .production_banked
            .binary_search_by(|b| b.item.cmp(&item))
        {
            Ok(i) => self.production_banked.remove(i).progress,
            Err(_) => 0,
        }
    }

    /// Progress `item` would have if the city selected it now: its own stored progress when
    /// active, else its bank plus any unassigned Production.
    pub fn production_for(&self, item: ProductionItem) -> i32 {
        if self.producing == Some(item) {
            return self.production_stored;
        }
        if matches!(item, ProductionItem::Project { .. }) {
            return 0;
        }
        self.banked(item)
            + if self.holds_progress() {
                0
            } else {
                self.production_stored
            }
    }

    /// Change the active build. The old unit or building keeps its progress in the bank;
    /// the new one resumes its own bank. Unassigned Production (overflow, idle banking)
    /// joins the new unit or building. Returns the previous build.
    pub fn switch_production(&mut self, next: Option<ProductionItem>) -> Option<ProductionItem> {
        self.switch_production_for(next, false)
    }

    /// Egypt's ordinary buildings draw only their own bank. Idle or overflow Production is
    /// discarded when one starts; other items retain the usual Production banks.
    pub fn switch_production_for(
        &mut self,
        next: Option<ProductionItem>,
        egypt: bool,
    ) -> Option<ProductionItem> {
        if self.producing == next {
            return next;
        }
        self.forfeit_purchase();
        let previous = std::mem::replace(&mut self.producing, next);
        let mut carried = self.production_stored;
        if let Some(
            old @ (ProductionItem::Unit { .. }
            | ProductionItem::Building { .. }
            | ProductionItem::Wonder { .. }
            | ProductionItem::Improvement { .. }),
        ) = previous
        {
            self.bank(old, carried);
            carried = 0;
        }
        if egypt && matches!(next, Some(ProductionItem::Building { .. })) {
            carried = 0;
        }
        self.production_stored = carried
            + match next {
                Some(
                    item @ (ProductionItem::Unit { .. }
                    | ProductionItem::Building { .. }
                    | ProductionItem::Wonder { .. }
                    | ProductionItem::Improvement { .. }),
                ) => self.take_banked(item),
                _ => 0,
            };
        previous
    }

    /// Cancel the purchase while retaining all production earned normally.
    pub fn forfeit_purchase(&mut self) {
        if let Some(bought) = self.bought.take() {
            self.production_stored -= bought.progress;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityMemory {
    pub id: CityId,
    pub owner: PlayerId,
    pub name: String,
    pub population: i32,
    pub hp: i32,
    pub walls: bool,
    #[serde(default)]
    pub citadel: bool,
    #[serde(default)]
    pub star_fort: bool,
    /// Public observation of the Workshop line, used to reconstruct a city under fog.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub has_workshop: bool,
    /// Public observation of an Air Defence (`BuildingKind::AirDefence`): an attacker's forecast
    /// shows it, as it shows Walls.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub has_air_defence: bool,
    pub is_capital: bool,
}

impl CityMemory {
    /// The Walls tier above plain Walls: 0 none, 1 Castle, 2 Star Fort.
    pub fn fort_tier(&self) -> u8 {
        u8::from(self.citadel) + u8::from(self.star_fort)
    }
}

impl From<&City> for CityMemory {
    fn from(city: &City) -> Self {
        Self {
            id: city.id,
            owner: city.owner,
            name: city.name.clone(),
            population: city.population,
            hp: city.hp,
            walls: crate::data::has_building(&city.buildings, BuildingKind::Walls),
            citadel: crate::data::has_building(&city.buildings, BuildingKind::Castle),
            star_fort: crate::data::has_building(&city.buildings, BuildingKind::StarFort),
            has_workshop: city
                .buildings
                .iter()
                .any(|building| crate::data::is_workshop_line_building(*building)),
            has_air_defence: crate::data::has_building(&city.buildings, BuildingKind::AirDefence),
            is_capital: city.is_capital,
        }
    }
}

impl CityMemory {
    /// Public identity is current on explored land; private city facts stay at their last
    /// observation. A newly founded city has no private observation yet.
    pub fn with_public_identity(current: &Self, previous: Option<&Self>) -> Self {
        let mut memory = previous
            .filter(|old| old.id == current.id)
            .cloned()
            .unwrap_or(Self {
                id: current.id,
                owner: current.owner,
                name: String::new(),
                population: 1,
                hp: crate::data::CITY_HP,
                walls: false,
                citadel: false,
                star_fort: false,
                has_workshop: false,
                has_air_defence: false,
                is_capital: false,
            });
        memory.owner = current.owner;
        memory.name.clone_from(&current.name);
        memory
    }
}

/// Last observation of mutable tile information. Terrain never changes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileMemory {
    pub owner: Option<PlayerId>,
    pub ruin: bool,
    /// The remembered tile's resource was improved (`Tile::improved`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub improved: bool,
    pub city: Option<CityMemory>,
}

/// A military unit's last visible location, never a claim about its current position.
/// Kept separately from live units so fogged observations cannot become combat targets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreatObservation {
    pub unit: UnitId,
    pub owner: PlayerId,
    pub kind: UnitKind,
    pub pos: Hex,
    pub seen_turn: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub faction: Faction,
    pub team: TeamId,
    pub is_human: bool,
    pub alive: bool,
    pub gold: i32,
    pub science_stored: i32,
    #[serde(default, skip_serializing_if = "science_conversion_is_gold")]
    pub science_conversion: ScienceConversion,
    pub researching: Option<TechId>,
    /// Ordered research plan, including `researching` at the front.
    #[serde(default)]
    pub research_queue: Vec<TechId>,
    pub techs: BTreeSet<TechId>,
    /// Tiles this player has ever seen (index = map index).
    pub explored: BitSet,
    /// Last visible observation by map index; None for tiles never observed.
    #[serde(default)]
    pub tile_memory: Vec<Option<TileMemory>>,
    /// Bounded recent observations, shared through team vision for humans and AI alike.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_threats: Vec<ThreatObservation>,
    /// Locations of this player's rejected founding attempts, without the hidden cause.
    /// Retained across turns and saves so a stateless AI cannot retry the same failed site.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub failed_city_sites: BTreeSet<Hex>,
    pub ruins_explored: u32,
    /// Settlers this empire has trained (raises the next settler's cost).
    pub settlers_trained: i32,
    /// Paid military reinforcements during this war. Current rules have permanent war.
    /// Owner-only history keeps stateless AI budgets stable across save/load.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub military_purchases: u32,
    /// Science generated in each of the last few turns, newest last (for "Most Advanced").
    pub science_history: Vec<i32>,
    /// Total yields (Food + Production + Gold + Science) of each of the last turns,
    /// newest last. Its average is the empire's public Power (`scoring::power`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub power_history: Vec<i32>,
    /// This empire's decadence in percent, the cut taken from every city yield
    /// (`scoring::decadence_percent`). Refreshed once per turn from its lead over the average, right
    /// after the turn's Power is recorded, so yields read one number and cost nothing to look up.
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub decadence_percent: i32,
    /// VP v2: fights this empire has won so far (`objectives`), counted from the game start.
    #[serde(
        default,
        skip_serializing_if = "crate::objectives::FightStats::is_zero"
    )]
    pub fight: crate::objectives::FightStats,
    /// VP v2: each era this empire has entered, from the second on (`objectives`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub eras: BTreeMap<u8, crate::objectives::EraRecord>,
    /// Rival original capitals this empire burned in a Celtic sack, with their names. Each keeps
    /// its 1 VP for the rest of the game (the city is gone, so nobody can take it back); only the
    /// Celtic sack path adds to it (`rules::battle::capture_city`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capitals_sacked: Vec<(CityId, String)>,
    /// Permanently chosen on completion; captured buildings never populate this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub great_work_completed: Option<BuildingKind>,
    /// Last holder scored for this empire's Great Work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub great_work_holder: Option<PlayerId>,
    /// The empire's own faction goal scores for it now (`goals`): a Permanent goal once earned,
    /// a Live goal while its condition holds.
    pub faction_goal_done: bool,
    /// The goal has scored at least once (a Live goal regained is not a first completion).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub faction_goal_reached_before: bool,
    /// Who scored this empire's Held goal at the last scoring (`goals::held_by`): the Heir's
    /// holder. Used to report each change of hands once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_holder: Option<PlayerId>,
    /// Mongols only: the city its Yam Relay Station was built in. The Station is a Held goal: the city
    /// can change hands (the Station goes with it and so does the goal), and razing it destroys the
    /// Station (`goals::hall_city` checks the city still stands with the Station in it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hall_city: Option<CityId>,
    /// India's Imperial Diwan stays with its city when captured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diwan_city: Option<CityId>,
    /// Free technologies a rival's Share the Signal broadcast left this empire to choose
    /// (`Action::ChooseFreeTechs`). The prompt persists across turns; nothing auto-picks.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub free_techs_pending: u8,
    /// Whose broadcast granted `free_techs_pending` (for the prompt's portrait).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub free_techs_from: Option<PlayerId>,
    /// Athens' Greece Captive: the captured city that granted the pending free technology
    /// (None for a broadcast), so the prompt names its real source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub free_techs_captured: Option<CityId>,
    /// Sparta goal, Honour the Fallen: veterans retired in Agoge cities so far
    /// (`Action::RetireVeteran`). Progress is kept; the goal is done at
    /// `goals::SPARTA_RETIRE_COUNT`.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub veterans_retired: u8,
    /// Japan's Sakoku vow (`Action::Sakoku`, `goals::GoalRule::Sakoku`): `Some(n)` while the
    /// country is closed, with the turn ends it has stayed closed (0 on the turn the vow is
    /// taken). Public: the closed state and the countdown are on every empire's view.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sakoku_turns: Option<u32>,
    /// A goal rule other than the faction's own (`goals::rule`). Scenarios and tests set it;
    /// `None` means the faction's default (`goals::default_rule`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_rule: Option<crate::goals::GoalRule>,
    /// Strategic supply users (Forges, Factories, Pikemen), sorted by user, with since when
    /// each has used supply and whether it is idle (`resources::update_supply`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supply: Vec<crate::resources::SupplyRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    /// Optional continuation-only RNG seed for dice. Absent in recorded games, so their
    /// serialized states and hashes retain the original game-seed stream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dice_seed: Option<u64>,
    /// Wonders that have paid their first-sighting reward, independent of current ownership.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub wonders_discovered: BTreeSet<Hex>,
    /// Camps, raids and camp memory; `None` when the game has barbarians off (and in saves
    /// from before barbarians existed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub barbarians: Option<crate::rules::barbarians::BarbarianState>,
    /// Absent only in saves from the legacy all-war ruleset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diplomacy: Option<crate::diplomacy::DiplomacyState>,
    pub setup: GameSetup,
    pub turn: u32,
    pub map: Map,
    pub players: Vec<Player>,
    pub units: BTreeMap<UnitId, Unit>,
    pub cities: BTreeMap<CityId, City>,
    pub next_unit_id: u32,
    pub next_city_id: u32,
    /// Set when the game ends; `finished` with no winner is a draw.
    pub winner: Option<TeamId>,
    pub finished: bool,
    /// Victory by VP is won by holding the target for `data::VP_HOLD_TURNS` turn ends: a team is listed
    /// here from the end of the turn it reaches its VP target (count 1), with the number of
    /// consecutive turn ends it has been at or above that target. It leaves the map the moment it drops below the target (or
    /// is eliminated).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub vp_holds: BTreeMap<TeamId, u32>,
    /// The first empire into each era (era index to player), set when it is entered
    /// (`Event::EraEntered`). Ancient has none: everyone starts there.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub era_firsts: BTreeMap<u8, PlayerId>,
    /// Aircraft (`rules::air`), each in one city's hangar; ordered by id. They are not map units:
    /// they never stand on a tile, are never garrison, and are lost with their city.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aircraft: Vec<Aircraft>,
    /// Air reconnaissance in force (`Order::AirRecon`, `rules::air`): each reveals the tiles around
    /// its target to its owner's team for the one turn after it was ordered.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub air_recons: Vec<AirRecon>,
    /// Aircraft seen in rival hangars or reported in battles, retained per player's team.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub air_contacts: BTreeMap<PlayerId, Vec<AirContact>>,
    /// The per-turn record behind the post-game report (`history::record`, appended by
    /// `resolve_turn`). Empty until a turn resolves; public only once the game is finished.
    #[serde(default, skip_serializing_if = "crate::history::GameHistory::is_empty")]
    pub history: crate::history::GameHistory,
}

impl GameState {
    /// Dice and stochastic turn resolution use this stream; map and objectives use setup.seed.
    pub fn dice_seed(&self) -> u64 {
        self.dice_seed.unwrap_or(self.setup.seed)
    }
}

/// An aircraft in a city's hangar (the Modern air layer, `UnitClass::Air`). It shares the unit id
/// space, so orders and events name it like a unit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aircraft {
    pub id: UnitId,
    pub kind: UnitKind,
    /// The city whose hangar it is in; its owner is the aircraft's owner.
    pub city: CityId,
    pub hp: i32,
    /// Cumulative combat experience (`rules::promotion`), kept through upgrades and rebasing.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub xp: u32,
}

/// One air reconnaissance (`Order::AirRecon`): the tiles within `data::RECON_RADIUS` of `target`
/// are in `owner`'s team's sight for the planning turn after `turn`, the turn it was ordered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AirRecon {
    pub owner: PlayerId,
    pub target: Hex,
    pub turn: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AirContact {
    pub owner: PlayerId,
    /// Exact kind only when a hangar was seen or the aircraft defended an interception.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<UnitKind>,
    pub turn: u32,
    /// Last seen hangar; absent when the aircraft was known only from a battle report.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<CityId>,
}

/// The read-only record behind `PlayerId::BARBARIAN`. Barbarian units live in
/// `GameState::units` so movement, combat and capture treat them like any hostile unit; this
/// record lets every rule that reads a unit owner's faction or team work unchanged. India has
/// no unit modifiers (combat already uses it as the neutral faction). `player_mut` has no
/// barbarian record, so nothing can give barbarians Gold, techs or VP.
static BARBARIAN_PLAYER: std::sync::LazyLock<Player> = std::sync::LazyLock::new(|| Player {
    id: PlayerId::BARBARIAN,
    name: "Barbarians".into(),
    faction: Faction::India,
    team: TeamId::BARBARIAN,
    is_human: false,
    alive: true,
    gold: 0,
    science_stored: 0,
    science_conversion: ScienceConversion::Gold,
    researching: None,
    research_queue: Vec::new(),
    techs: BTreeSet::new(),
    explored: BitSet::new(0),
    tile_memory: Vec::new(),
    observed_threats: Vec::new(),
    failed_city_sites: BTreeSet::new(),
    ruins_explored: 0,
    settlers_trained: 0,
    military_purchases: 0,
    science_history: Vec::new(),
    power_history: Vec::new(),
    decadence_percent: 0,
    fight: crate::objectives::FightStats::ZERO,
    eras: BTreeMap::new(),
    capitals_sacked: Vec::new(),
    great_work_completed: None,
    great_work_holder: None,
    faction_goal_done: false,
    goal_holder: None,
    hall_city: None,
    diwan_city: None,
    faction_goal_reached_before: false,
    free_techs_pending: 0,
    free_techs_from: None,
    free_techs_captured: None,
    veterans_retired: 0,
    sakoku_turns: None,
    goal_rule: None,
    // Barbarians have no economy: no supply.
    supply: Vec::new(),
});

impl GameState {
    pub fn unit_moves_for(&self, owner: PlayerId, kind: UnitKind) -> i32 {
        let player = self.player(owner);
        self.setup
            .rule_experiments
            .unit_moves_for(player.faction, kind)
    }

    /// The faction whose version of `u` it is: its seller for a hired Company (`company`), else
    /// its owner's own faction. The unit's own stat line, name and rules follow it; faction-wide
    /// rules follow the owner.
    pub fn made_by(&self, u: &Unit) -> Faction {
        u.origin.unwrap_or_else(|| self.player(u.owner).faction)
    }

    /// The stats of `u`, as its owner fields it (`made_by` for its own line).
    pub fn unit_stats_of(&self, u: &Unit) -> crate::data::UnitStats {
        crate::data::unit_stats_made_by_rules(
            self.setup.rule_experiments.faction_rules_off,
            self.player(u.owner).faction,
            self.made_by(u),
            u.kind,
        )
    }

    /// The moves `u` gets a turn.
    pub fn unit_moves_of(&self, u: &Unit) -> i32 {
        self.unit_stats_of(u).moves
    }

    pub fn player(&self, id: PlayerId) -> &Player {
        if id.is_barbarian() {
            return &BARBARIAN_PLAYER;
        }
        &self.players[id.0 as usize]
    }

    pub fn player_mut(&mut self, id: PlayerId) -> &mut Player {
        &mut self.players[id.0 as usize]
    }

    pub fn team_of(&self, id: PlayerId) -> TeamId {
        self.player(id).team
    }

    pub fn same_team(&self, a: PlayerId, b: PlayerId) -> bool {
        self.team_of(a) == self.team_of(b)
    }

    pub fn team_members(&self, team: TeamId) -> Vec<PlayerId> {
        self.players
            .iter()
            .filter(|p| p.team == team)
            .map(|p| p.id)
            .collect()
    }

    /// All team ids, sorted.
    pub fn teams(&self) -> Vec<TeamId> {
        let set: BTreeSet<TeamId> = self.players.iter().map(|p| p.team).collect();
        set.into_iter().collect()
    }

    pub fn units_at(&self, h: Hex) -> impl Iterator<Item = &Unit> {
        self.units.values().filter(move |u| u.pos == h)
    }

    pub fn city_at(&self, h: Hex) -> Option<&City> {
        self.cities.values().find(|c| c.pos == h)
    }

    pub fn cities_of(&self, p: PlayerId) -> impl Iterator<Item = &City> {
        self.cities.values().filter(move |c| c.owner == p)
    }

    pub fn units_of(&self, p: PlayerId) -> impl Iterator<Item = &Unit> {
        self.units.values().filter(move |u| u.owner == p)
    }

    pub fn capital_of(&self, p: PlayerId) -> Option<&City> {
        self.cities_of(p).find(|c| c.is_capital)
    }

    pub fn alloc_unit_id(&mut self) -> UnitId {
        self.next_unit_id = self
            .next_unit_id
            .checked_add(1)
            .expect("unit IDs exhausted");
        UnitId(self.next_unit_id)
    }

    pub fn alloc_city_id(&mut self) -> CityId {
        self.next_city_id = self
            .next_city_id
            .checked_add(1)
            .expect("city IDs exhausted");
        CityId(self.next_city_id)
    }

    /// FNV-1a 64 over the canonical JSON serialization. Equal states hash equally everywhere.
    /// The JSON is streamed into the hash, never held in memory: the bytes and so the hash are
    /// exactly those of `serde_json::to_vec(self)`.
    pub fn hash(&self) -> u64 {
        let mut hasher = Fnv1a(FNV_OFFSET);
        serde_json::to_writer(&mut hasher, self).expect("state serializes");
        hasher.0
    }
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv1a_update(mut h: u64, bytes: &[u8]) -> u64 {
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// An `io::Write` that folds every byte written into an FNV-1a 64 state.
struct Fnv1a(u64);

impl std::io::Write for Fnv1a {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = fnv1a_update(self.0, bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn fnv1a(bytes: &[u8]) -> u64 {
    fnv1a_update(FNV_OFFSET, bytes)
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

fn science_conversion_is_gold(value: &ScienceConversion) -> bool {
    *value == ScienceConversion::Gold
}

fn is_zero_u8(v: &u8) -> bool {
    *v == 0
}

fn is_zero_i32(v: &i32) -> bool {
    *v == 0
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streamed_hash_equals_hash_of_the_serialized_bytes() {
        let state = crate::new_game(&GameSetup::all_ai(7));
        let (next, _) = crate::resolve_turn(&state, &BTreeMap::new());
        for s in [&state, &next] {
            let bytes = serde_json::to_vec(s).unwrap();
            assert_eq!(s.hash(), fnv1a(&bytes));
        }
        assert_eq!(fnv1a(b""), FNV_OFFSET);
    }
}

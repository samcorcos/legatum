// Mirrors the JSON the Rust core produces (crates/core/src/{types,view,orders,events,forecast,
// economy,state}.rs). serde conventions: snake_case fields, unit-like enums as strings, Order and
// Event tagged by "type", ProductionItem and Defender tagged by "kind", ids as plain numbers,
// tuples as 2-element arrays, 64-bit seeds and hashes as decimal strings.

export type PlayerId = number;
export type TeamId = number;
export type UnitId = number;
export type CityId = number;

export interface Hex {
  q: number;
  r: number;
}

export type Terrain =
  | 'Ocean'
  | 'Coast'
  | 'Grassland'
  | 'Plains'
  | 'Forest'
  | 'Hills'
  | 'Desert'
  | 'Tundra'
  | 'Mountains';

export type Faction = 'Switzerland' | 'India' | 'Byzantium' | 'Germany' | 'Celts' | 'Mongols' | 'Sparta' | 'Athens' | 'Aztecs' | 'Egypt' | 'Rome';
export type UnitKind =
  | 'Scout' | 'Settler' | 'Spearman' | 'Archer' | 'Pikeman' | 'Crossbowman' | 'Musketman' | 'HeirToTheThrone'
  | 'Horseman' | 'Knight' | 'Cavalry' | 'Tank' | 'AutonomousArmour'
  | 'Rifleman' | 'Infantry' | 'HunterKiller' | 'StealthFighter' | 'StealthBomber'
  | 'FieldCannon' | 'Artillery' | 'CombatDrone'
  | 'Fighter' | 'Bomber'
  | 'Phalangite' | 'CompositeArcher' | 'Cataphract' | 'Arquebusier' | 'Cuirassier'
  | 'TrenchInfantry' | 'Howitzer' | 'EarlyTank' | 'Biplane'
  | 'MechanisedInfantry' | 'MainBattleTank' | 'RocketArtillery' | 'JetFighter' | 'JetBomber';
/** A military unit's class (core `UnitClass`): melee beats mounted, ranged beats melee, mounted beats ranged. */
export type UnitClass = 'Melee' | 'Ranged' | 'Mounted' | 'Air';
export type AttackMode = 'Melee' | 'Ranged';
/** Core `forecast::AlongsideAttack`: another of your units already ordered to attack the same tile this turn. */
export interface AlongsideAttack { unit: number; mode: AttackMode }
/** Core `forecast::AssaultGarrison`: the garrison's share of the combined assault. */
interface AssaultGarrison {
  unit: number;
  owner: number;
  kind: UnitKind;
  hp: number;
  /** Expected HP left after this turn's attacks. */
  hp_after: number;
  /** 0-100: chance the garrison is destroyed. */
  destroyed_chance: number;
  /** Expected HP the garrison loses to the carried part of the hit that empties the city's HP (core `combat::carried_share`). */
  carried?: number;
}
/**
 * Core `forecast::CityAssaultForecast`: the whole fight at an enemy city, the city and its garrison
 * together, with every attack you have queued on it this turn (damage pooled by stage, excess reaching
 * the garrison). Numbers come from core; the card only words them.
 */
export interface CityAssaultForecast {
  /** Attacks pooled, this one included. */
  attacks: number;
  city_hp: number;
  city_hp_after: number;
  garrison?: AssaultGarrison | null;
  /** Core `forecast::AssaultSupporter`: units of the city's owner assumed to be ordered into it (supported defence); empty unless named. */
  supporters?: { unit: number; kind: UnitKind; hp: number; hp_after: number; destroyed_chance: number }[];
  /** Expected damage this attacker takes from the pooled counterattack of every defender. */
  reply?: number;
  /** 0-100: chance the city is captured this turn; 0 when no melee attack is queued. */
  capture_chance: number;
  /** A melee attack is among the pooled ones. */
  can_capture: boolean;
}
export type BuildingKind =
  | 'RiceStorehouse'
  | 'Granary'
  | 'Workshop'
  | 'Market'
  | 'Library'
  | 'Walls'
  | 'Mill'
  | 'Forge'
  | 'Agoge'
  | 'University'
  | 'Bank'
  | 'Factory'
  | 'Castle'
  | 'StarFort'
  | 'GreatLibrary'
  | 'Pyramids'
  | 'HagiaSophia'
  | 'EiffelTower'
  | 'IterFusionReactor'
  | 'RoyalArsenal'
  | 'GrandBazaar'
  | 'ResearchLab'
  | 'YamRelayStation'
  | 'ImperialDiwan'
  | 'Roadworks'
  | 'StockExchange'
  | 'Supermarket'
  | 'AirDefence';
/** Declaration order is research order: five per era, nine eras (core `eras::TECHS`). `Cannonry` is the
 * Industrial-era "Ballistics". */
export type TechId =
  | 'Irrigation'
  | 'BronzeWorking'
  | 'Masonry'
  | 'Currency'
  | 'Writing'
  | 'Mathematics'
  | 'Engineering'
  | 'IronWorking'
  | 'CodeOfLaws'
  | 'Philosophy'
  | 'Feudalism'
  | 'Machinery'
  | 'Chivalry'
  | 'Education'
  | 'CropRotation'
  | 'Gunpowder'
  | 'Metallurgy'
  | 'MilitaryEngineering'
  | 'Banking'
  | 'PrintingPress'
  | 'Rifling'
  | 'MilitaryScience'
  | 'Cannonry'
  | 'Industrialization'
  | 'Railways'
  | 'ReplaceableParts'
  | 'Combustion'
  | 'Steel'
  | 'Flight'
  | 'Electricity'
  | 'MassMobilisation'
  | 'Rocketry'
  | 'ArmouredWarfare'
  | 'AirPower'
  | 'Electronics'
  | 'MechanisedWarfare'
  | 'CompositeArmour'
  | 'GuidedMissiles'
  | 'JetEngines'
  | 'GreenRevolution'
  | 'AutonomousSystems'
  | 'Robotics'
  | 'FusionPower'
  | 'StealthTechnology'
  | 'RecursiveAi';
export type CityFocus = 'Balanced' | 'Growth' | 'Production';

export type ProductionItem =
  | { kind: 'Unit'; unit: UnitKind }
  | { kind: 'Building'; building: BuildingKind }
  | { kind: 'Wonder'; building: BuildingKind; tile: Hex }
  | { kind: 'Project'; project: 'Gold' | 'Science' | 'Food' | 'Grow' }
  /** Improve one owned, worked resource tile (core `improvements`); `improvement` matches the tile's resource. */
  | { kind: 'Improvement'; improvement: ImprovementKind; tile: Hex };

/** A tile improvement (core `ImprovementKind`); it upgrades by itself with technology. */
export type ImprovementKind = 'Farm' | 'Pasture' | 'Quarry' | 'Mine' | 'SaltpetreWorks' | 'OilWell' | 'QuartzMine';

export interface Yields {
  food: number;
  production: number;
  gold: number;
  science: number;
}

/** A number with the parts that make it up. `parts` are [label, amount] pairs. */
export interface Breakdown {
  total: number;
  parts: [string, number][];
}

interface YieldSource {
  label: string;
  yields: Yields;
}

export interface CityYields {
  total: Yields;
  sources: YieldSource[];
  worked: Hex[];
  /** Read-only priorities derived from the build and population. */
  focus: CityFocus;
  focus_reason: string;
}

// ---- view.rs ----

export type Wonder = 'MountFuji' | 'VictoriaFalls' | 'GrandCanyon' | 'GiantsCauseway' |
  'Uluru' | 'HaLongBay' | 'IguazuFalls' | 'GreatBarrierReef';

export interface TileView {
  explored: boolean;
  visible: boolean;
  terrain: Terrain | null;
  ruin: boolean;
  wonder?: Wonder | null;
  /** Public built world wonder or capturable goal building on this exact tile. */
  landmark?: { building: BuildingKind; city: CityId; of: PlayerId } | null;
  /** The wonder's one-time discovery reward has been paid, to some empire. Absent = still open. */
  wonder_found?: boolean;
  owner: PlayerId | null;
  /** A resource the viewer knows (G11): bonus ones on explored tiles, strategic ones once revealed. */
  resource?: Resource | null;
  /** A built improvement on the tile's resource, last seen. */
  improvement?: TileImprovement | null;
  /** The tile's road links (core `Tile::road`): bit d links to the neighbour in direction d, 64 marks a road tile. Public and always current on explored tiles. Absent = no road. */
  road?: number;
}

/** A built improvement as the viewer sees it; the upgraded form shows only on the viewer's own tiles. */
interface TileImprovement {
  kind: ImprovementKind;
  upgraded?: boolean;
}

/** What building an improvement does, shown with its production option. */
export interface ImprovementPreview {
  tile: Hex;
  resource: Resource;
  /** What the tile gains when built, at the owner's current technology. */
  gain: Yields;
  /** The automatic upgrade the owner doesn't have yet. */
  upgrade?: { tech: TechId; name: string; gain: Yields } | null;
}
export interface WonderPreview { tile: Hex; lost: Yields }

// ---- resources.rs (G11) ----

export type Resource = 'Wheat' | 'Cattle' | 'Stone' | 'Copper' | 'Iron' | 'Niter' | 'Coal' | 'Oil' | 'Silicon';
export type ResourceClass = 'Bonus' | 'Strategic';

/** One supplying deposit (core `resources::DepositView`). */
interface DepositView {
  hex: Hex;
  city: CityId | null;
  mined: boolean;
  supply: number;
}

/** One strategic supply as its owner sees it (the top bar chip "Iron 1/6" spare/total). */
export interface SupplyView {
  resource: Resource;
  revealed_by: TechId | null;
  revealed: boolean;
  /** What the deposits give, one line per deposit ("Iron near Lugano, with a Mine": 3). */
  supplied: Breakdown;
  /** The same deposits as data, so the tooltip names each by its tile. Omitted when empty. */
  deposits?: DepositView[];
  /** What uses it, one line per building or group of units ("Forge in Lugano": 1). */
  used: Breakdown;
  /** Users short of supply, newest first: they yield nothing (buildings) or fight weaker (units). */
  idle?: string[];
  /** Builds in progress holding supply for when they finish ("Forge in Bern"). */
  held?: string[];
  /** Supply left for one more user: supplied − used − held. */
  spare: number;
}

/** A building tier yielding nothing for lack of strategic supply. */
interface IdleBuilding {
  building: BuildingKind;
  resource: Resource;
}

export type Automation = 'Explore';
/** Military and Economy pick builds; Gold, Science and Food run that project (core `CityBuildFocus`). */
export type CityBuildFocus = 'Military' | 'Economy' | 'Gold' | 'Science' | 'Food';
export type StandingOrder = 'Sleep' | 'Heal' | 'AutoFire';
export type ExploreStopReason = 'Complete' | 'Threatened' | 'Damaged' | 'Blocked';

/** Core `PursuitOutcome`: how an attack order on a unit ended this turn (it lasts one turn, G18). */
export type PursuitOutcome = 'Followed' | 'Attacked' | 'LostSight' | 'AwaitingWar' | 'OutOfReach';

export interface UnitView {
  id: UnitId;
  owner: PlayerId;
  kind: UnitKind;
  pos: Hex;
  hp: number;
  moves_left: number;
  max_moves: number;
  /** Only for this player's own units. */
  goto: Hex | null;
  path?: Hex[];
  automation?: Automation | null;
  standing_order?: StandingOrder | null;
  /** Owner-only request for a new decision, even while fortified. */
  needs_orders?: boolean;
  /** Owner-only: the enemy unit this unit's attack order pursues (core `attack_target`). */
  attack_target?: UnitId;
  fortify: number;
  strength: number;
  ranged: number;
  range: number;
  military: boolean;
  /** Promotion level (0..5, or 6 for Sparta), public and shown as enlisted insignia. */
  level?: number;
  /** Own military units only: cumulative combat experience. */
  xp?: number | null;
  /** Own military units only: cumulative XP for the next level; absent at the maximum level. */
  xp_next?: number | null;
  /** Own units only, occupation lock: the civ this unit is locked in its city against. */
  occupation_lock?: PlayerId | null;
  /** Gold to upgrade now, if possible. */
  upgrade_cost: number | null;
  upgrade_to?: UnitKind | null;
  upgrade_steps?: UnitKind[];
  upgrade_breakdown?: Breakdown | null;
  /** Own units: the strategic supply the upgrade also uses (Phalangite→Pikeman: 1 Iron). */
  upgrade_need?: Resource | null;
  /** Any visible unit short of the supply it needs: it fights weaker until supply returns. */
  unsupplied?: Resource | null;
  /** The strength an unsupplied unit loses in every fight: show `strength - unsupplied_loss`, "27, was 36: no Iron". */
  unsupplied_loss?: number | null;
  /** Own units from a save made before the era rule: they need no supply until upgraded. */
  /** Own units: why the upgrade can't take the supply it needs now (the build card's sentence). */
  upgrade_supply_blocker?: string | null;
  /** The unit has bought Siege (core `siege`): its attacks on a city ignore the Walls its owner's technology reaches. Public, like its gear on the map. */
  siege?: boolean;
  /** Own land military units without Siege: its Gold price, and why it cannot be bought now (technology, Mongol era, territory, Gold, Iron). */
  siege_offer?: { price: Breakdown; blocker: string | null } | null;
  sight?: number | null;
  resting_heal?: Breakdown | null;
  /** An Heir only (public): the Byzantium empire whose goal he carries, whoever holds him now. */
  heir_of?: PlayerId | null;
  /** A hired Mercenary Company (public): the last turn of its contract (it goes home at the end of that turn) and the faction that sold it. */
  contract_until?: number;
  origin?: Faction;
}

/** Military upkeep effect of building one more of this unit (legatum-ui: every build card shows
 * its ongoing cost). Absent for civilian units, buildings and projects, which have no upkeep. */
export interface UpkeepPreview {
  /** True while the empire still has a free military slot for this unit. */
  free: boolean;
  /** Free military slots left before this unit (0 once all are used). */
  free_slots_left: number;
  /** Gold/turn this unit costs once paid (1, or 3 for the Switzerland). */
  rate: number;
  /** The empire's net Gold/turn after building this unit. */
  gold_per_turn_after: number;
}

/** Core `economy::BuyBlock`, serialized snake_case. */
export type BuyBlock = 'already_bought' | 'resistance' | 'settler_population' | 'not_buyable' | 'walls_need_production' | 'no_spawn_tile' | 'resource' | 'not_enough_gold' | 'egypt_buildings_need_food' | 'under_threat';

export interface ProductionOption {
  item: ProductionItem;
  cost: number;
  /** Progress this item would have if selected now: its own progress when active, else its
   * banked progress plus any unassigned Production (overflow, idle banking). */
  production_stored: number;
  /** Progress banked for this item from an earlier switch (0 for the active build). */
  production_banked: number;
  buy_cost: number | null;
  buy_blocker?: string | null;
  /** Why buying is blocked, as data (core `economy::BuyBlock`); absent when it isn't blocked. */
  buy_block?: BuyBlock;
  buy_quote?: { production: Breakdown; gold_percent: number; additional_settler_production: number } | null;
  yields: CityYields;
  turns: number;
  /** Strategic supply the item uses (1 of this resource) for as long as it exists. */
  supply_need?: Resource | null;
  /** Why it can't start now for lack of that supply, with how to get more. It can still be queued. */
  resource_blocker?: string | null;
  /** Military upkeep effect of building this now; absent for civilians, buildings and projects. */
  upkeep?: UpkeepPreview | null;
  /** For an Improvement item: its tile and what it adds. */
  improvement?: ImprovementPreview | null;
  /** The current tile income this wonder replaces on completion. */
  wonder?: WonderPreview | null;
}

/** Progress a city put into an item it switched away from; it waits for that item only. */
interface BankedProduction {
  item: ProductionItem;
  progress: number;
}

/** A researched item the city can't choose or queue right now, and why (shown greyed out). */
export interface UnavailableOption {
  item: ProductionItem;
  cost: number;
  reason: string;
  /** Missing in-city building predecessor, provided by core when this is an upgrade path. */
  prerequisite_building?: BuildingKind | null;
  production_banked: number;
}

export interface CityDetail {
  /** This city's founder differs from its current owner. */
  captured?: boolean;
  original_capital?: boolean;
  former_owner?: PlayerId;
  /** Core's reason Raze is unavailable; absent when legal. */
  raze_blocker?: string;
  yields: CityYields;
  idle_yields?: CityYields | null;
  food_stored: number;
  growth_threshold: number;
  growth_breakdown?: Breakdown;
  turns_to_grow: number | null;
  producing: ProductionItem | null;
  production_queue?: ProductionItem[];
  /** A rival completed the world wonder this city was building; it waits for a new choice (core `City.wonder_lost`). */
  wonder_lost?: WonderLoss | null;
  build_automation?: CityBuildFocus | null;
  last_auto_pick?: ProductionItem | null;
  auto_queued?: ProductionItem | null;
  production_stored: number;
  production_cost: number | null;
  turns_to_complete: number | null;
  /** Why an Egyptian building pauses and growth resumes. */
  food_construction_paused?: string | null;
  production_stall: ProductionStall | null;
  idle_production_cap: number;
  /** Lost next turn if this city stays idle. */
  idle_production_waste: number;
  buy_cost: number | null;
  buildings: BuildingKind[];
  /** Building tiers idle for lack of strategic supply: they yield nothing until it returns. */
  idle_buildings?: IdleBuilding[];
  /** Buildings from a save made before the era rule: they need no supply. */
  options: ProductionOption[];
  /** Researched units and buildings this city can't choose now (Settler at population 1,
   * population gates, missing lower tier), with the reason. world wonders are not listed. */
  unavailable: UnavailableOption[];
  /** Banked progress per item, sorted by item. */
  production_banked: BankedProduction[];
  /** Every tile the city owns, its centre included. Those within the work radius are worked. */
  territory: Hex[];
  /** The tile the city claims when it next grows; null when no unowned tile touches its territory. */
  next_claim: ClaimPreview | null;
  /** The price before the ring surcharge: a tile costs this times `tileRingPercent(ring)`. */
  tile_base_price: number;
  /** Explored unowned tiles touching this city's borders within its work radius, nearest ring first, each with its Gold price. */
  buyable_tiles: BuyableTile[];
  /** Why this city can buy no tile this turn (purchase used, occupation, resistance, no tile in reach), or null. Gold is per tile. */
  buy_tile_blocker?: string | null;
  resistance: number;
  /** Turns until starvation costs a population; absent while fed, resisting or at population 1. */
  turns_to_starve?: number | null;
  /** Turns until the city is back to full HP if nothing damages it, at 20 HP per turn; absent at full HP. */
  turns_to_heal?: number | null;
  /** An at-war military unit is adjacent, so the city regains no HP at the end of this turn; absent otherwise. */
  heal_blocked?: boolean;
  /** The city took battle damage on the turn that just resolved (own cities only); absent otherwise. Survives save and load, unlike the turn report. */
  attacked_last_turn?: boolean;
  /** Items this city makes only from a population (Settler at 2, upgrade buildings), reached or not. */
  population_unlocks?: PopulationUnlock[];
}

/** The tile a city's next growth claims, with what it adds. */
/** A tile a city can buy, with its ring (distance from the city) and Gold price. */
export interface BuyableTile {
  tile: Hex;
  ring: number;
  price: number;
}

interface ClaimPreview {
  tile: Hex;
  yields: Yields;
  /** Within the work radius: worked as soon as it is claimed. Otherwise it only blocks rivals. */
  worked: boolean;
}

interface PopulationUnlock {
  population: number;
  item: ProductionItem;
}

export interface CityView {
  id: CityId;
  owner: PlayerId;
  name: string;
  pos: Hex;
  population: number;
  hp: number;
  max_hp: number;
  strength: number;
  is_capital: boolean;
  original_capital?: boolean;
  original_capital_of?: PlayerId | null;
  founder?: PlayerId;
  capture_bounty?: number | null;
  greece_captive_tech?: boolean;
  greece_captive_at_risk?: boolean;
  walls: boolean;
  /** A Castle or better. */
  citadel?: boolean;
  star_fort?: boolean;
  /** Whether the last visible observation showed a Workshop or its upgrade. */
  has_workshop?: boolean;
  /** Whether the last visible observation showed an Air Defence: it intercepts air strikes on this city (core `BuildingKind::AirDefence`). */
  has_air_defence?: boolean;
  /** Occupation lock: the civ a military unit of yours would be locked against by ending a move in this city. */
  occupation_lock?: PlayerId | null;
  /** Completed world wonders are public in every explored city, including through fog. */
  great_works: BuildingKind[];
  /** Public chosen wonder tile and current construction stage, including through fog. */
  construction?: { building: BuildingKind; tile: Hex; stage: number; progress: number } | null;
  /** The city is building Walls: public, including through fog, like a wonder site. */
  walls_building?: boolean;
  visible: boolean;
  detail: CityDetail | null;
}

/** One way a player scored: the core's `types::VpSource`. A held goal (a capturable item) scores for whoever holds it, so the goal names its empire (`of`). */
export type VpSource = { kind: 'GreatWork'; of: PlayerId } | { kind: 'FactionGoal'; of: PlayerId } | { kind: 'EraObjective'; era: number } | { kind: 'CapitalTaken'; capital: CityId } | { kind: 'CapitalSacked'; capital: CityId };

/** Core `goals::GoalRule`: which rule a goal follows (the Aztecs use the Swarm rule for the New Fire). */
export type GoalRule = 'None' | 'Hoard' | 'Swarm' | 'Sakoku' | 'Diwan' | 'Heir' | 'Factories' | 'Wild' | 'Retire' | 'Hall' | 'Broadcast' | 'Colonia' | 'Tomb';

export interface PlayerPublic {
  id: PlayerId;
  name: string;
  faction: Faction;
  /** Public game setup; an omitted difficulty means Normal in older views. */
  difficulty?: AiDifficulty;
  search?: boolean;
  team: TeamId;
  is_human: boolean;
  alive: boolean;
  vp: number;
  /** What the VP are made of, by id: era objectives, captured capitals, Great Works and faction goals. */
  vp_sources: [VpSource, number][];
  capitals_taken?: [CityId, string][];
  /** Celts only: rival original capitals this empire sacked, each 1 VP kept for good (core `capitals_sacked`). */
  capitals_sacked?: [CityId, string][];
  /** Its original capital still stands in its own team's hands (public; core `scoring::capital_stands`). Absent in older views: standing. */
  capital_stands?: boolean;
  cities: number;
  faction_goal_done: boolean;
  /** Sakoku (Japan): the country is closed, with the turn ends it has been closed (public). Null while open. */
  sakoku_turns?: number | null;
  /** Egypt is building its Royal Tomb (public): none of its cities builds or grows while it does. Absent when not. */
  tomb_work?: boolean;
  /** Turn ends of work this empire has put toward its Royal Tomb (public; kept across a pause, absent at 0). */
  tomb_turns?: number;
  /** The rule this empire's goal follows (core `goals::GoalRule`; public). */
  goal_rule?: GoalRule | null;
  /** The empire's current era, 1-based (1 Ancient .. 8 Future; core `objectives::era_of`): it rises the turn the previous era's last technology lands. Public. */
  era: number;
  /** Thin levies (Byzantium only, public): armed units fielded and the cap (one per city, plus one). */
  levy?: LevyView;
  /** Mercenary Companies this empire fields (public; absent when none). */
  companies?: number;
  /** Its current Gold, public to every player (0 once dead). */
  gold?: number;
  /** Public economy value: Gold plus Production across its cities (0 once dead). */
  economy?: number;
  /** Public army strength: combat value of its units and aircraft (0 once dead). */
  army?: number;
  /** Public technological progress: furthest era entered (times 1000) plus technologies known (0 once dead). */
  advancement?: number;
  /** The trailing average of this empire's total yields (Food + Production + Gold + Science). */
  power?: number;
  /** Its Power over the mean living empire's, in percent (negative below it). */
  lead_percent?: number;
  /** Optional decadence: the percent cut from every yield of this empire's cities. */
  decadence_percent?: number;
}

/** One seller's Company in the shop (core `company::Offer`): its own special unit, what it costs now in Gold, and why it cannot be hired yet, if so. */
export interface CompanyOffer { seller: PlayerId; unit: UnitKind; price: number; unavailable?: string }

/** An empire's armed units against its thin-levy cap (core `view::LevyView`). */
export interface LevyView { units: number; cap: number }

interface EconomyView {
  military_purchases?: number;
  gold: number;
  gold_per_turn: Breakdown;
  science_per_turn: Breakdown;
  upkeep: number;
  upkeep_breakdown?: Breakdown;
  /** Turns until the treasury can't pay upkeep and a unit deserts (1 = this turn's end). Only within 5 turns. */
  bankruptcy: BankruptcyForecast | null;
}

export type ProductionStall =
  | { kind: 'Population'; required: number; available: number }
  | { kind: 'SpawnBlocked' };

export interface BankruptcyForecast {
  /** Treasury plus income minus upkeep, before resetting a negative treasury to zero. */
  gold: Breakdown;
  disband: { unit: UnitId | null; kind: UnitKind; pos: Hex; city: CityId | null } | null;
}

export interface TechEffects {
  upgrades: { unit: UnitId; from: UnitKind; to: UnitKind; before: number; after: Breakdown; eligible: boolean }[];
}
export interface TechOption {
  effects?: TechEffects;
  tech: TechId;
  cost: Breakdown;
  turns: number | null;
  /** 0-based era the technology belongs to. */
  era: number;
  /** Unknown and in the empire's current era: the strict gate. */
  researchable: boolean;
  income_change?: YieldSource[];
  /** Other living empires that know it (its diffusion discount). */
  known_by: number;
}

export interface TechUnlock {
  tech: TechId;
  item: ProductionItem;
  name: string;
  cost: number;
  buy_cost: number | null;
  yields: Yields;
  city_strength: number;
  unit_strength: number;
  upgrade_cost: number | null;
  cost_breakdown?: Breakdown;
  unit_ranged?: number;
  unit_range?: number;
  unit_moves?: number;
  siege_bonus?: number;
  upgrade_from?: UnitKind | null;
  predecessor?: BuildingKind | null;
  min_population?: number;
  /** Core `research::locked_note`: why this faction cannot build the item yet ("Walls for the Mongols from the Medieval era"); said in place of the build line and prices. */
  locked_note?: string | null;
  cities?: { city: CityId; blocker: string | null; build_turns: number | null; yields_after: CityYields; food_surplus_after: number; turns_to_six_after?: number | null; turns_to_eight_at_gate?: number | null; turns_to_eight_after?: number | null }[];
}

/** One queued technology's arrival forecast, in queue order. */
export interface ScheduleSlot {
  tech: TechId;
  cost: number;
  /** Turns from now until it completes (after earlier queued items); null with no Science income. */
  turns: number | null;
  arrival_turn: number | null;
}
export interface ResearchView {
  science_conversion?: 'Gold' | 'Food' | 'Production';
  /** The empire's current era, 0 (Ancient) to 6 (Future). Only this era's technologies can be researched. */
  era: number;
  era_name: string;
  era_total: number;
  era_known: number;
  /** The first empire into each era, public: [era, empire]. */
  era_firsts: [number, PlayerId][];
  schedule: ScheduleSlot[];
  current: TechId | null;
  /** Ordered plan, including current. Optional for older views. */
  queue?: TechId[];
  progress: number;
  cost: number | null;
  turns_left: number | null;
  known: TechId[];
  /** Techs this faction skips: known from the start because they give it nothing. Research screens leave them out; `era_total` and `era_known` exclude them. */
  skipped?: TechId[];
  available: TechOption[];
  catalog: TechOption[];
  unlocks: TechUnlock[];
}

/** VP v2: the families an era objective can come from. */
export type ObjectiveFamily = 'Build' | 'Expand' | 'Fight';

/** One public era objective (VP v2); `slot` is 0 Build, 1 Expand, 2 Fight, or in a World Age game always 0 (one card per Age). */
interface ObjectiveView {
  slot: number;
  family: ObjectiveFamily;
  text: string;
  target: number;
  /** The rule behind the text, as the engine serializes it (the AI keys on it; the app only echoes it back to the engine). */
  objective: { kind: string; [field: string]: unknown };
}

/** VP v2: an era's public objectives and who has entered and scored it. Era 1 has none, so the list starts at era 2. */
export interface EraView {
  era: number;
  name: string;
  objectives: ObjectiveView[];
  entered_by: PlayerId[];
  scored_by: { player: PlayerId; slot: number; turn: number }[];
}

/** VP v2: the viewer's own progress on one era objective (eras entered, plus the next one). */
export interface ObjectiveProgress {
  era: number;
  slot: number;
  current: number;
  target: number;
  entered: boolean;
  era_scored: boolean;
  /** World Ages: the turn the Age began. */
  entered_turn?: number | null;
  ready?: boolean | null;
}

/** Own gain-count totals and the other open eras affected by scoring. */
export interface ObjectiveAccounting {
  era: number;
  slot: number;
  baseline: number;
  total: number;
  counting_since_turn: number;
  restart_eras: number[];
}

/** A team that has reached its VP target wins once it has held it this many consecutive turns (10 turns); dropping below resets the count. Public. */
interface VictoryHold {
  /** Consecutive turn ends at or above the target, counting the turn it was reached (1 then). */
  turns_held: number;
  turns_needed: number;
}

export interface TeamStanding {
  team: TeamId;
  members: PlayerId[];
  vp: number;
  target: number;
  /** Present while the team holds its VP target; it wins when `turns_held` reaches `turns_needed`. */
  victory_hold?: VictoryHold | null;
}

export interface Relation {
  a: TeamId;
  b: TeamId;
  state: 'Peace' | 'War' | 'Alliance';
  security?: SecurityTreaty | null;
  effective_turn: number;
  guarantee_until: number;
  version: number;
  war_id: number | null;
  declarers: TeamId[];
}
export type TreatyKind = 'Peace' | 'Alliance';
/** The terms of an active Alliance. Leaving before `protected_until` costs Gold and is a betrayal. */
interface SecurityTreaty { id: number; started_turn: number; protected_until: number }
/** `by` left an Alliance with `against` inside its protected window, or declared war on it while allied. */
export interface Betrayal { by: TeamId; against: TeamId; turn: number; until: number }
interface DecisionExplanation { score: number | null; reasons: string[] }
export interface PeaceProposal {
  explanation?: DecisionExplanation;
  kind?: TreatyKind;
  id: number;
  from: TeamId;
  to: TeamId;
  proposer: PlayerId;
  created_turn: number;
  respond_by: number;
  relation_version: number;
  /** Peace only: the teams that have agreed so far, the proposer first. A Peace binds only its two named teams. */
  consents?: TeamId[];
  /** Views only: the teams whose answer is still awaited (never the proposer). */
  needs?: TeamId[];
  /** Views and Peace events only: the core's two named teams, including one that already agreed. */
  required_teams?: TeamId[];
}
/** A war that begins for `team` against `enemy` because Alliances make each side one. */
export interface NewWar { team: TeamId; enemy: TeamId }
interface SupersededOrder {
  team: TeamId;
  player: PlayerId;
  order: Order;
  winner: PlayerId;
  winner_order: Order;
  reported_turn: number;
}
export interface DiplomaticAction {
  order: Order;
  effective_turn: number;
  /** One-off Gold from the ordering empire for leaving an Alliance early. */
  cost: number;
  /** Gold per turn each empire on both teams pays while a signed Alliance lasts. */
  upkeep?: number;
  /** Leaves an Alliance inside its protected window, or declares war on an ally. */
  betrayal?: boolean;
  /** The wars this action starts besides the one it names: for an Alliance, the wars either partner already defends (the other joins at once); for a declaration, the allies of the target who defend it. */
  new_wars?: NewWar[];
  /** The calls to arms this action raises: for a declaration, the allies asked to join; for an Alliance, the wars either partner started, asked of the other. */
  calls?: NewWar[];
  /** Peace: the other teams whose agreement the whole Peace still needs. */
  also_agree?: TeamId[];
  blocked_reason: string | null;
}
/** A call to arms (core `diplomacy::WarCall`): `ally` declared war on `target` and `team`, its ally, may join or decline. Answered with `RespondToProposal` and the call's `id`. */
export interface WarCall {
  id: number;
  team: TeamId;
  ally: TeamId;
  target: TeamId;
  created_turn: number;
  respond_by: number;
  alliance_version: number;
  war_version: number;
}
export interface DiplomacyView {
  /** Calls to arms waiting for this team's answer. */
  calls?: WarCall[];
  /** Public: every betrayal still refusing Alliances. */
  betrayals?: Betrayal[];
  exploration?: { native_land: number; borrowed_current: number };
  replies?: { proposal: PeaceProposal; accepted: boolean; effective_turn: number }[];
  queued: { player: PlayerId; order: Order }[];
  legacy: boolean;
  relations: Relation[];
  /** Every Alliance side of two or more teams (public). */
  alliances?: TeamId[][];
  proposals: PeaceProposal[];
  superseded: SupersededOrder[];
  peace_cooldowns: { from: TeamId; to: TeamId; retry_turn: number; kind?: TreatyKind }[];
  /** Other teams this team has met (core `DiplomacyView::met`): an Alliance needs contact. */
  met?: TeamId[];
  actions: DiplomaticAction[];
}

interface BarbarianCampView {
  pos: Hex;
  /** Currently in sight (otherwise city identity and owner are current, private facts remembered). */
  visible: boolean;
}

/** One city's share of the New Fire (core `events::SwarmLoss`). */
interface SwarmLoss { city: CityId; lost: number; population: number; tiles: number }

/** One city's part of the New Fire, previewed (core `rules::actions::SwarmCityView`). */
export interface SwarmCityView { city: CityId; lost: number; population: number; tiles: Hex[] }
interface SwarmView {
  /** Population sacrificed. */
  population: number;
  /** Population the empire can spare now (every city keeps 1). */
  spare: number;
  /** Why the New Fire can't be ordered now. */
  blocker: string | null;
  cities: SwarmCityView[];
}
/** Japan's Sakoku vow (core `rules::actions::SakokuView`; own view only). */
interface SakokuView {
  closed: boolean;
  /** Turn ends spent closed (0 while open). */
  turns: number;
  /** Turn ends the country must stay closed before it holds its VP. */
  target: number;
  /** The VP scores now: the countdown is done. */
  holding: boolean;
  /** Why the next move (closing, or reopening) can't be ordered now. */
  blocker: string | null;
}
/** Egypt's Royal Tomb (core `rules::actions::TombView`; own view only). */
export interface TombView {
  /** The work is on: no city builds or grows. */
  working: boolean;
  /** Turn ends counted toward the tomb (kept across a pause). */
  turns: number;
  /** Turn ends the tomb needs. */
  target: number;
  /** The city the tomb would stand in if the work finished now (the capital, else the largest city). */
  site: CityId | null;
  /** The tomb's city, once it stands (in anyone's hands). */
  tomb: CityId | null;
  /** Why the next move (starting, or pausing while it is on) can't be ordered now. */
  blocker: string | null;
}
/** Rome's Colonia Victrix while none stands (core `rules::actions::ColoniaFoundersView`; own view only). */
export interface ColoniaFoundersView {
  /** The promotion level the founder needs. */
  level: number;
  /** The highest level among the empire's melee units. */
  best: number;
  /** Melee veterans of that level, in id order: each founds the Colonia on a site that counts. */
  founders: UnitId[];
}
interface BroadcastView { techs_each: number; blocker: string | null; receivers: PlayerId[] }
/** Free technologies the viewer has to choose (a rival's broadcast). */
interface FreeTechsView {
  pending: number;
  from: PlayerId | null;
  /** Athens' Greece Captive: the captured city that granted them; null for a broadcast. */
  captured?: CityId | null;
  /** Technologies pickable now (prerequisites known); a pick can unlock more. */
  choices: TechId[];
}
/** A veteran the Sparta could retire (core `rules::actions::RetireVeteranView`). */
export interface RetireVeteranView {
  unit: UnitId;
  level: number;
  /** Why he can't retire where he stands; the action is offered only when null. */
  blocker: string | null;
}
/** The Sparta's Honour the Fallen (core `rules::actions::RetireView`). */
interface RetireView {
  retired: number;
  /** Veterans to retire. */
  target: number;
  /** The promotion level a veteran needs. */
  level: number;
  /** Own cities with an Agoge: where veterans retire. */
  shrines: CityId[];
  /** Every unit of the required level, in id order. */
  veterans: RetireVeteranView[];
}
/** One city's wild camp as Call the warbands would raise it now (core `WildSiteView`). */
interface WildSiteView { city: CityId; at: Hex }
/** The Celts's Call the warbands (core `rules::actions::WildView`). */
interface WildView {
  blocker: string | null;
  /** The camps the call would raise now, one per city with room. */
  sites: WildSiteView[];
  /** Cities with a camp in their territory now, and cities owned. */
  with_camp: number;
  cities: number;
}

/** The viewer's one-time actions (core `rules::actions::ActionsView`). */
export interface ActionsView {
  swarm?: SwarmView | null;
  sakoku?: SakokuView | null;
  /** Egypt: the Royal Tomb (absent for every other empire). */
  tomb?: TombView | null;
  /** Rome: present while its goal is open (no Colonia Victrix stands). */
  colonia?: ColoniaFoundersView | null;
  broadcast?: BroadcastView | null;
  free_techs?: FreeTechsView | null;
  retire?: RetireView | null;
  wild?: WildView | null;
}

/** Core-calculated own-unit contributions to the army strength. */
export interface ArmyContribution {
  unit: UnitId;
  kind: UnitKind;
  strength: number;
  ranged: number;
  level: number;
  unsupplied: boolean;
  promotion_bonus: number;
  supply_penalty: number;
  combat_strength: number;
  full_health: number;
  hp: number;
  value: number;
}

export interface PlayerView {
  army_breakdown?: ArmyContribution[];
  rule_experiments?: RuleSwitches;
  /** The game started with the AI-only teams allied in pairs. Omitted when off. */
  ai_alliances?: boolean;
  diplomacy: DiplomacyView;
  turn: number;
  turn_limit: number | null;
  me: PlayerId;
  team: TeamId;
  width: number;
  height: number;
  /** Row-major (odd-r offset), index = row * width + col. */
  tiles: TileView[];
  units: UnitView[];
  cities: CityView[];
  failed_city_sites?: Hex[];
  /** Explored, otherwise-foundable tiles inside the real founding gap of a city; no city identity or center. */
  city_gap_blocked?: Hex[];
  /** Barbarian camps this team sees (`visible`) or remembers. Absent with barbarians off. */
  barbarian_camps?: BarbarianCampView[];
  players: PlayerPublic[];
  economy: EconomyView;
  /** Strategic supply, one per strategic resource, revealed or not. */
  supply?: SupplyView[];
  research: ResearchView;
  /** Every era after the first, with its public objectives. */
  eras: EraView[];
  /** VP v2: your own progress on the objectives of the eras you have entered and the next one. */
  objective_progress: ObjectiveProgress[];
  objective_accounting?: ObjectiveAccounting[];
  development?: DevelopmentForecast | null;
  /** Your own faction goal's progress, for every faction: none once a Permanent goal is done, always there for a Live or Held one. Rivals' progress is private. */
  faction_goal?: FactionGoalView | null;
  /** The viewer's one-time actions and pending free technologies. */
  actions?: ActionsView;
  /** Every standing Yam Relay Station (public): whose it is, who holds it now (the owner of its city) and where. Its holder's land is open to every empire at Peace. */
  courier_halls?: CourierHallView[];
  imperial_diwans?: ImperialDiwanView[];
  /** Every standing Colonia Victrix (public, through fog and on unexplored land): whose goal it is, who holds it, its city, place and name. */
  colonias?: GoalCityView[];
  /** Every standing Royal Tomb (public, as `colonias`). */
  royal_tombs?: GoalCityView[];
  /** Where each Royal Tomb under construction will stand (core `goals::tomb_site`), one per empire whose work is on (public). `holder` is the city's owner. */
  tomb_sites?: GoalCityView[];
  /** Your team's aircraft, each in one city's hangar (never on the map). */
  aircraft?: AircraftView[];
  /** Rival aircraft currently visible in a city's hangar; no private health or supply. */
  air_sightings?: AirSightingView[];
  /** Aircraft previously seen in hangars or reported in a battle; the base may be unknown. */
  air_contacts?: AirContactView[];
  /** Byzantium only, private: every Company it could hire now, by seller and kind (the shop). Sellers at War, unmet or on its team are already left out. */
  company_offers?: CompanyOffer[];
  great_works?: GreatWorkView[];
  standings: TeamStanding[];
  winner: TeamId | null;
  finished: boolean;
  /** The game's public record (core `history::GameHistory`), present only once the game is finished. */
  history?: GameHistory | null;
}

// ---- history.rs ----

/** One record per empire per resolved turn plus the landmark events (core `history`). Entry i of a
 *  series describes the state after turn `first_turn + i`. Shown only after the game ends. */
export interface GameHistory {
  first_turn: number;
  /** Indexed by player id. */
  empires: EmpireHistory[];
  chronicle?: Chronicle[];
}

export interface EmpireHistory {
  vp: number[];
  cities: number[];
  population: number[];
  /** Science produced that turn. */
  science: number[];
  /** Treasury after the turn. */
  gold: number[];
  /** The Strongest Army measure (combat value scaled by health). */
  army: number[];
  techs: number[];
  /** 1-based era; absent for history recovered from an older save's archive. */
  era?: number[];
}

export type Chronicle =
  | { kind: 'Founded'; turn: number; player: PlayerId; city: string }
  | { kind: 'Captured'; turn: number; by: PlayerId; from: PlayerId; city: string }
  | { kind: 'Razed'; turn: number; by: PlayerId; from: PlayerId; city: string; ordered?: boolean }
  | { kind: 'War'; turn: number; a: TeamId; b: TeamId; by?: TeamId | null }
  | { kind: 'Peace'; turn: number; a: TeamId; b: TeamId }
  | { kind: 'Era'; turn: number; player: PlayerId; era: number; first: boolean }
  | { kind: 'GreatWork'; turn: number; player: PlayerId; building: BuildingKind; city: string }
  | { kind: 'Wonder'; turn: number; player: PlayerId; wonder: Wonder }
  | { kind: 'Objective'; turn: number; player: PlayerId; era: number; objective: string }
  | { kind: 'Goal'; turn: number; player: PlayerId }
  | { kind: 'Eliminated'; turn: number; player: PlayerId }
  | { kind: 'HoldStarted'; turn: number; team: TeamId }
  | { kind: 'HoldBroken'; turn: number; team: TeamId }
  | { kind: 'Won'; turn: number; team: TeamId; reason: string }
  | { kind: 'Drawn'; turn: number };

// ---- orders.rs ----

/** What an open air picker is choosing: a strike target, a rebase city or a tile to scout. */
export type AirPick = 'strike' | 'rebase' | 'recon';

export type Order =
  | { type: 'DeclareWar'; against: TeamId; breach?: boolean }
  /** Byzantium only, at War with `to`: pay the quoted price and begin an enforced Peace this turn. */
  | { type: 'BreakTreaty'; with: TeamId; agreement: 'Security' }
  | { type: 'ProposeTreaty'; to: TeamId; kind: TreatyKind }
  | { type: 'RespondToProposal'; proposal: number; accept: boolean; explanation?: DecisionExplanation }
  | { type: 'WithdrawProposal'; proposal: number }
  | { type: 'SetAutomation'; unit: UnitId; automation: Automation | null; path?: Hex[]; stop_reason?: ExploreStopReason | null }
  | { type: 'SetAutoFire'; unit: UnitId; enabled: boolean; target?: Hex | null }
  | { type: 'MoveUnit'; unit: UnitId; to: Hex }
  | { type: 'MoveUnitPath'; unit: UnitId; to: Hex; path: Hex[]; stop_on_block?: boolean }
  /** `until_healed`: rest like Sleep, then wake and ask for orders the turn HP is full. */
  | { type: 'SleepUnit'; unit: UnitId; until_healed?: boolean }
  | { type: 'SkipUnit'; unit: UnitId }
  | { type: 'FoundCity'; unit: UnitId }
  | { type: 'DisbandUnit'; unit: UnitId }
  | { type: 'Raze'; city: CityId }
  | { type: 'SetProduction'; city: CityId; item: ProductionItem }
  | { type: 'SetCityAutomation'; city: CityId; focus: CityBuildFocus | null; picked?: ProductionItem | null }
  | { type: 'SetProductionQueue'; city: CityId; queue: ProductionItem[] }
  | { type: 'BuyItem'; city: CityId; item: ProductionItem }
  | { type: 'SetResearch'; tech: TechId }
  | { type: 'SetResearchQueue'; techs: TechId[] }
  | { type: 'SetScienceConversion'; to: 'Gold' | 'Food' | 'Production' }
  | { type: 'RangedAttack'; unit: UnitId; target: Hex }
  /** An aircraft in a city's hangar strikes a tile within its range: a Bomber's 5, a Fighter's 3 (core `rules::air`). A striking Fighter does not intercept that turn. */
  | { type: 'AirStrike'; plane: UnitId; target: Hex }
  /** An aircraft scouts a tile within its strike range: the tiles within `RULES.reconRadius` of it are visible next turn only, so a strike can be ordered then (core `rules::air`). It uses the aircraft's one order. */
  | { type: 'AirRecon'; plane: UnitId; target: Hex }
  /** An aircraft moves to another own city's hangar. */
  | { type: 'Rebase'; plane: UnitId; to: CityId }
  /** `to`: the line where there is a choice (a Biplane: 'Fighter' or 'Bomber'); omitted for the default line. */
  | { type: 'UpgradeAircraft'; plane: UnitId; to?: UnitKind }
  | { type: 'UpgradeUnit'; unit: UnitId }
  /** Byzantium only: hire a Mercenary Company of `unit` from `seller` in `city` (core `company`); it takes the city's one purchase for the turn. */
  | { type: 'HireCompany'; city: CityId; unit: UnitKind; seller: PlayerId }
  | { type: 'BuyTile'; city: CityId; tile: Hex }
  /** Buy Siege for a land military unit in own territory: its attacks on a city ignore enemy Walls, for good (core `rules::siege`). */
  | { type: 'UpgradeSiege'; unit: UnitId }
  | { type: 'Action'; action: GameAction }
  /** Give Gold to a living teammate (core `rules::gifts`): it moves after this turn's purchases, capped at the Gold left then. */
  | { type: 'GiveGold'; to: PlayerId; amount: number };

/**
 * A one-time empire action (core `orders::Action`, serde tag "action"). One order carries them
 * all: `{ type: 'Action', action: { action: 'Swarm' } }`.
 */
export type GameAction =
  /** The New Fire (Swarm): offer `goals::AZTEC_NEW_FIRE_POPULATION`, biggest cities first. */
  | { action: 'Swarm' }
  /** Japan, Sakoku: close the country (`close: true`) or reopen it. */
  | { action: 'Sakoku'; close: boolean }
  /** Egypt, the Royal Tomb: start (or resume) the work (`build: true`) or pause it. */
  | { action: 'Tomb'; build: boolean }
  /** Athens, Share the Learning: every rival outside the team gains free techs of its choice. */
  | { action: 'Broadcast' }
  /** Spend a broadcast's free technologies (1 to the pending count, prerequisites known or listed first). */
  | { action: 'ChooseFreeTechs'; techs: TechId[] }
  /** Sparta, Honour the Fallen: retire a veteran standing in an own Agoge city. */
  | { action: 'RetireVeteran'; unit: UnitId }
  /** Celts, Call the warbands: raise a barbarian camp in the territory of every Celts city. */
  | { action: 'CallTheWild' };


export interface OrderError {
  /** Index into the submitted orders. */
  index: number;
  message: string;
}

// ---- events.rs ----

/** Who fired or struck in a Battle: a unit id, or a city's automatic shot (untagged in core). */
export type BattleAttacker = UnitId | { city: CityId };

/** The attacking unit's id, or null when a city fired. */
export const attackerUnit = (a: BattleAttacker): UnitId | null => (typeof a === 'number' ? a : null);

type Defender =
  | { kind: 'Unit'; unit: UnitId; owner: PlayerId; unit_kind: UnitKind }
  | { kind: 'City'; city: CityId; owner: PlayerId };

interface RuinResearch {
  tech: TechId; cost: number; progress_before: number; income: number;
  turns_saved: number | null; completes: boolean;
}
export type RuinReward =
  | { kind: 'Gold'; amount: number; balance: number; affordable: { city: CityId; item: ProductionItem; cost: number } | null }
  | { kind: 'Science'; amount: number; research: RuinResearch | null }
  | { kind: 'Population'; city: CityId; amount: number; population: number; yields: Yields }
  | { kind: 'FreeScout'; unit: UnitId };

export type Event =
  | { type: 'WonderDiscovered'; owner: PlayerId; at: Hex; wonder: Wonder; gold: number; science: number }
  | { type: 'WarDeclared' | 'PeaceEstablished'; relation: Relation }
  /** An Alliance carried `team` into `ally`'s war: `relation` is `team`'s war with the enemy. */
  | { type: 'WarJoined'; relation: Relation; team: TeamId; ally: TeamId }
  | { type: 'CallToArms'; call: WarCall }
  | { type: 'CallAnswered'; call: WarCall; joined: boolean; effective_turn: number }
  | { type: 'CallLapsed'; call: WarCall; effective_turn: number; reason: string }
  /** Byzantium's Heir was born in `city` (public: every empire is told, and his position is public from then on). */
  | { type: 'HeirBorn'; city: CityId; owner: PlayerId; city_name: string; at: Hex }
  /** Byzantium hired a Mercenary Company: `gold` went to `seller`, and the unit serves until the end of turn `until`. */
  | { type: 'CompanyHired'; city: CityId; owner: PlayerId; seller: PlayerId; unit: UnitKind; gold: number; until: number }
  /** A Company's contract ended and it went home from `pos`. */
  | { type: 'CompanyDeparted'; unit: UnitKind; owner: PlayerId; origin?: Faction | null; pos: Hex }
  /** An unpaid Company turned barbarian where it stood. */
  | { type: 'CompanyTurned'; unit: UnitKind; owner: PlayerId; origin?: Faction | null; pos: Hex }
  | { type: 'TreatyActivated'; relation: Relation }
  | { type: 'TreatyEnded'; treaty_id: number; a: TeamId; b: TeamId; effective_turn: number; reason: string }
  | { type: 'TreatyBreached'; treaty_id: number; by: TeamId; with: TeamId; player: PlayerId; cost: number; effective_turn: number }
  | { type: 'TreatyPayment'; player: PlayerId; gold: number; cause: string; treaty_id: number | null; effective_turn: number }
  | { type: 'AllianceBetrayed'; betrayal: Betrayal }
  | { type: 'ProposalCreated'; proposal: PeaceProposal }
  | { type: 'ProposalAccepted' | 'ProposalRejected' | 'ProposalWithdrawn' | 'ProposalExpired'; proposal: PeaceProposal; effective_turn: number }
  | { type: 'ProposalInvalidated'; proposal: PeaceProposal; effective_turn: number; reason: string }
  | { type: 'OrderSuperseded'; decision: SupersededOrder }
  | { type: 'UnitAwakened'; unit: UnitId; owner: PlayerId; unit_kind: UnitKind; at: Hex }
  | { type: 'UnitHealed'; unit: UnitId; owner: PlayerId; unit_kind: UnitKind; at: Hex }
  | { type: 'AutomationStopped'; unit: UnitId; owner: PlayerId; at: Hex; reason: ExploreStopReason }
  | { type: 'UnitMoved'; unit: UnitId; owner: PlayerId; from: Hex | null; to: Hex | null }
  | { type: 'MoveBlocked'; unit: UnitId; owner: PlayerId; at: Hex | null; toward: Hex | null; reason: string }
  | { type: 'CityReinforced'; unit: UnitId; owner: PlayerId; unit_kind: UnitKind; city: CityId; at: Hex }
  | { type: 'Bounce'; units: UnitId[]; tile: Hex }
  | { type: 'AttackPursuit'; unit: UnitId; owner: PlayerId; at: Hex; target: UnitId; target_owner: PlayerId; target_kind: UnitKind; target_at: Hex | null; outcome: PursuitOutcome }
  | {
      type: 'Battle';
      /** A unit id, or `{ city }` for a city's automatic shot (cities fire back). */
      attacker: BattleAttacker;
      attacker_owner: PlayerId;
      defender: Defender;
      at: Hex;
      /**
       * Where the attacker stood: the shooter's or striking city's tile, or the tile a melee attacker
       * struck from. Absent when the viewer's team saw neither that tile nor the attacker's side.
       */
      attacker_at?: Hex;
      attacker_strength: Breakdown;
      defender_strength: Breakdown;
      damage_to_attacker: number;
      damage_to_defender: number;
      attacker_roll: number;
      defender_roll: number;
      attacker_hp_after: number;
      defender_hp_after: number;
      ranged: boolean;
      /** The report identifies an aircraft attack, but may hide its exact kind. */
      air_strike?: boolean;
      meeting: boolean;
      /** A melee hit on a unit moving away (G21): no fortification; if it survived, it went on. */
      defender_leaving?: boolean;
    }
  | { type: 'UnitDestroyed'; unit: UnitId; owner: PlayerId; unit_kind: UnitKind; at: Hex }
  | { type: 'UnitDisbanded'; unit: UnitId; owner: PlayerId; reason: string; }
  | { type: 'SettlerCaptured'; unit: UnitId; from: PlayerId; to: PlayerId; at: Hex }
  | { type: 'CityFounded'; city: CityId; owner: PlayerId; name: string; at: Hex }
  | { type: 'CityGrew'; city: CityId; owner: PlayerId; population: number }
  | { type: 'CityStarved'; city: CityId; owner: PlayerId; population: number }
  | { type: 'CityCaptured'; city: CityId; from: PlayerId; to: PlayerId; bounty?: number }
  | { type: 'AthensCityCaptured'; city: CityId; from: PlayerId; to: PlayerId }
  | { type: 'CapitalTaken'; captor: PlayerId; capital: CityId; name: string }
  | { type: 'CapitalSacked'; captor: PlayerId; capital: CityId; name: string }
  /** Team only: Gold one empire gave a teammate, after this turn's purchases. */
  | { type: 'GoldGiven'; from: PlayerId; to: PlayerId; amount: number }
  | { type: 'CapitalLost'; player: PlayerId; capital: CityId; name: string }
  | { type: 'CityRazed'; city: CityId; from: PlayerId; by: PlayerId; at: Hex; original_capital?: boolean; sack_gold?: number; camp?: Hex; ordered?: boolean }
  | { type: 'BuildingDismantled'; city: CityId; owner: PlayerId; building: BuildingKind; reason: string }
  | { type: 'ItemBought'; city: CityId; owner: PlayerId; item: ProductionItem; gold: number }
  | { type: 'ProductionCompleted'; city: CityId; owner: PlayerId; item: ProductionItem }
  | { type: 'MegaprojectStarted'; city: CityId; owner: PlayerId; building: BuildingKind }
  | { type: 'TechResearched'; player: PlayerId; tech: TechId }
  /** An empire entered an era, the turn the previous era's last technology landed (public). `era` is 1-based (2 Classical .. 8 Future); `first`: first empire into it. */
  | { type: 'EraEntered'; player: PlayerId; era: number; first: boolean }
  | { type: 'ResearchQueueAdvanced'; player: PlayerId; next: TechId | null }
  | { type: 'TileClaimed'; city: CityId; owner: PlayerId; tile: Hex }
  | { type: 'TileReleased'; city: CityId; owner: PlayerId; tile: Hex }
  | { type: 'TileBought'; city: CityId; owner: PlayerId; tile: Hex; gold: number }
  | { type: 'TilesReassigned'; from: CityId; to: CityId; owner: PlayerId; tiles: Hex[] }
  | { type: 'AircraftRebased'; plane: UnitId; owner: PlayerId; kind: UnitKind; from: CityId; to: CityId }
  | { type: 'RuinExplored'; unit: UnitId; owner: PlayerId; at: Hex; reward: RuinReward }
  | { type: 'GreatWorkCompleted'; player: PlayerId; of: PlayerId; building: BuildingKind; vp: number; again: boolean }
  | { type: 'GreatWorkLost'; player: PlayerId; of: PlayerId; building: BuildingKind; vp: number }
  /** `active`: the city was building it and stopped; otherwise the wonder was only queued or banked there. */
  | { type: 'WonderRaceLost'; city: CityId; owner: PlayerId; winner: PlayerId; building: BuildingKind; production: number; gold: number; active?: boolean }
  /** VP v2: an empire met one of `era`'s objectives (`slot` 0 Build, 1 Expand, 2 Fight) and earned its 1 permanent VP. Public. */
  | { type: 'EraObjectiveScored'; player: PlayerId; era: number; slot: number; objective: string }
  /** World Ages: an Age began, so its card is open to every empire. `era` is 1-based; `first` entered it first. Public. */
  | { type: 'AgeBegan'; era: number; first: PlayerId }
  /** `again`: a Live goal regained (absent on a first completion). */
  | { type: 'FactionGoalCompleted'; player: PlayerId; faction: Faction; vp: number; again?: boolean }
  | { type: 'FactionGoalLost'; player: PlayerId; faction: Faction; vp: number }
  | { type: 'FlowerWars'; city: CityId; owner: PlayerId; population: number }
  | { type: 'SwarmUnleashed'; player: PlayerId; cities: SwarmLoss[] }
  /** A legacy Sakoku scenario closed the country. Public. */
  | { type: 'SakokuClosed'; player: PlayerId }
  /** India reopened the country after `turns` turn ends closed. Public. */
  | { type: 'SakokuReopened'; player: PlayerId; turns: number }
  /** Egypt started (or resumed) work on its Royal Tomb, with `turns` already counted. Public. */
  | { type: 'TombStarted'; player: PlayerId; turns: number }
  /** Egypt paused work on its Royal Tomb with `turns` counted (the count is kept). Public. */
  | { type: 'TombPaused'; player: PlayerId; turns: number }
  /** The Royal Tomb stands in `city`: 2 VP to whoever holds it. Public. */
  | { type: 'TombRaised'; player: PlayerId; city: CityId; name: string; at: Hex }
  | { type: 'SignalBroadcast'; from: PlayerId; receivers: PlayerId[]; techs_each: number }
  | { type: 'RoadsLaid'; owner: PlayerId; city: CityId; at: Hex; linked: number }
  | { type: 'VeteranRetired'; player: PlayerId; unit: UnitId; kind: UnitKind; city: CityId; retired: number; of: number }
  | { type: 'WildCalled'; player: PlayerId; camps: number }
  | { type: 'FreeTechsChosen'; player: PlayerId; from: PlayerId | null; techs: TechId[] }
  | { type: 'UnitUpgraded'; unit: UnitId; owner: PlayerId; to: UnitKind; gold: number }
  /** A unit was given Siege (core `Event::SiegeBought`). */
  | { type: 'SiegeBought'; unit: UnitId; owner: PlayerId; gold: number }
  | { type: 'BarbarianCampSpawned'; camp: number; at: Hex }
  | { type: 'BarbarianCampDiscovered'; camp: number; team: TeamId; at: Hex }
  | { type: 'BarbarianScoutSpotted'; unit: UnitId; city: CityId; owner: PlayerId; at: Hex }
  | { type: 'BarbarianRaidLaunched'; camp: number; city: CityId; owner: PlayerId; direction: string }
  | { type: 'BarbarianCampCleared'; camp: number; at: Hex; by: PlayerId; unit: UnitId | null; gold: number }
  /** A strategic source became usable near `city` (or, with no city, a tech revealed it with none in your borders). */
  | { type: 'ResourcesRevealed'; player: PlayerId; resource: Resource; city: CityId | null; sources: number; supply: number }
  /** The number of your buildings and units idle for lack of `resource` changed (0: supply covers every user again). */
  | { type: 'SupplyShortage'; player: PlayerId; resource: Resource; idle: number; supplied: number; used: number }
  | { type: 'UnitPromoted'; unit: UnitId; owner: PlayerId; unit_kind: UnitKind; level: number; at: Hex }
  | { type: 'SpartaKillHeal'; player: PlayerId; unit: UnitId; victim_owner: PlayerId; unit_kind: UnitKind; hp_healed: number }
  | { type: 'SpartaConquest'; city: CityId; player: PlayerId; units_healed: number }
  | { type: 'OrderFailed'; player: PlayerId; reason: string }
  | { type: 'PlayerEliminated'; player: PlayerId }
  /** A team reached its VP target and now has to hold it for `turns_needed` turns. Public to every player. */
  | { type: 'VictoryHoldStarted'; team: TeamId; vp: number; target: number; turns_needed: number }
  /** A team that was holding its VP target dropped below it (or was eliminated): its count is back to zero. Public. */
  | { type: 'VictoryHoldBroken'; team: TeamId; turns_held: number; vp: number; target: number }
  | { type: 'GameWon'; team: TeamId; reason: string }
  | { type: 'GameDrawn'; reason: string };


// ---- forecast.rs ----

export interface BattleForecast {
  ranged: boolean;
  attacker_strength: Breakdown;
  defender_strength: Breakdown;
  /** 0-100: chance the defender is destroyed (or a city drops to 0 HP). */
  win_chance: number;
  /** 0-100: chance the attacker is destroyed. */
  loss_chance: number;
  expected_damage_to_attacker: number;
  expected_damage_to_defender: number;
  attacker_hp: number;
  defender_hp: number;
  /** The attacked city fires back this turn: present when the attacker starts within two tiles. */
  city_strike?: CityStrikeForecast;
  /**
   * The attacked city's visible garrison (core `combat::city_tile_defender`). `shielded`: the city
   * has HP, takes this attack, and the garrison takes no damage. Otherwise the garrison is the
   * defender, and a melee win against it captures the city.
   */
  garrison?: GarrisonForecast;
  /** Gold a Celtic attacker takes by capturing this city: 60 per population, nothing else (core `economy::celts_sack_gold`). */
  sack_gold?: number;
  /** What the city's Walls do to this attack (core `WallsForecast`); absent when the target is not a walled city. */
  walls?: WallsForecast;
  /** The attacker's own share of its damage to the city's HP, before Walls, when not all of it (core `data::city_hp_damage_percent`: a Biplane 50). */
  city_hp_percent?: number | null;
  /** An air strike on a city with an Air Defence: the strike is met and does not land (core `forecast::AirDefenceForecast`); absent for a Stealth Bomber, which passes it. */
  air_defence?: AirDefenceForecast;
}

/** Core `forecast::AirDefenceForecast`: the duel an Air Defence fights in a Fighter's place. The Air Defence takes no damage. */
export interface AirDefenceForecast {
  interceptor_strength: number;
  striker_strength: number;
  expected_damage_to_attacker: number;
  loss_chance: number;
}

/** Core `forecast::WallsForecast`: the city's walls tier (1 Walls, 2 a Castle, 3 a Star Fort), the siege tier of the attacker (0 when it has no Siege) and the percent of its damage the attack deals to the city's HP. `built_in`: the attacker is an Industrial or later land unit, whose Siege is built in (core `data::built_in_siege`). */
interface WallsForecast {
  tier: number;
  covered: number;
  percent: number;
  built_in?: boolean;
}

interface GarrisonForecast {
  unit: UnitId;
  owner: PlayerId;
  kind: UnitKind;
  hp: number;
  shielded: boolean;
}

/**
 * A city's automatic shot in the ranged phase, at a 100% roll (it rolls 80-120%). Each city fires
 * once per turn at the visible hostile military unit within two tiles with the lowest HP.
 */
export interface CityStrikeForecast {
  city: CityId;
  city_strength: Breakdown;
  /** The attacker's defense where it stands now. */
  attacker_defense: Breakdown;
  damage: number;
}

// ---- state.rs (setup) ----

export type AiDifficulty = 'Easy' | 'Normal' | 'Hard';

export interface PlayerSetup {
  name: string;
  faction: Faction;
  team: TeamId;
  is_human: boolean;
  /** AI players only; omitted means Normal. */
  difficulty?: AiDifficulty;
  /** Training (AI at Hard only): its battles are searched. Omitted when false. */
  search?: boolean;
}

export interface GameSetup {
  /** u64 as a decimal string. */
  seed: string;
  width: number;
  height: number;
  players: PlayerSetup[];
  turn_limit: number | null;
  /** Legacy all-war start (Proxy benchmark only); omitted when false. */
  legacy_all_war?: boolean;
  /** Civ 6 style barbarian camps. Omitted (off) in setups from before barbarians existed. */
  barbarians?: boolean;
  /** The teams with no human seat start in Alliances of two (core `GameSetup::ai_alliances`). Omitted when off. */
  ai_alliances?: boolean;
  /** Core rule experiments (only the switches the app reads); omitted when all default. */
  rule_experiments?: RuleSwitches;
}

/** The core rule switches the app reads. */
export interface RuleSwitches {
  start_met?: boolean;
  start_settlers?: number;
  walls_cost?: number;
  decadence_slope?: number; vp_target?: number;
  /** Experiment: a city can't buy a military unit while an enemy unit at War is within 2 tiles. */
  no_purchase_under_threat?: boolean;
}

/** What end_turn returns. `turn` is the turn just resolved; `hash` is a u64 as a decimal string. */
export interface TurnResult {
  turn: number;
  events: Event[];
  /** Local clients filter separately for each human player. Online events are already filtered. */
  events_by_player?: Record<PlayerId, Event[]>;
  hash: string;
  finished: boolean;
  winner: TeamId | null;
  /** Training mode: log lines from the resolution (human positions, realised outcomes). */
  training_lines?: string[];
}

export interface DevelopmentForecast {
  cities: number;
  tall_cities: number;
  growth: { city: CityId; population: number; turns_to_six?: number | null; turns_to_eight: number | null }[];
  /** First simultaneous pair at population 8+ if current yield conditions hold; null beyond the game's expected length from now. */
  tall_turns: number | null;
}
/** An aircraft in a city's hangar (core `view::AircraftView`). */
/** Core `view::AircraftUpgradeOption`: a line the aircraft can upgrade along now (`via`, the order's `to`), the form it becomes and the Gold. */
export interface AircraftUpgradeOption { via: UnitKind; to: UnitKind; cost: number }
export interface AircraftView { id: UnitId; owner: PlayerId; kind: UnitKind; city: CityId; hp: number; level?: number; xp?: number | null; xp_next?: number | null; upgrade_cost?: number | null; upgrade_options?: AircraftUpgradeOption[]; unsupplied?: Resource | null }
export interface AirSightingView { owner: PlayerId; kind: UnitKind; city: CityId }
export interface AirContactView { owner: PlayerId; kind?: UnitKind | null; turn: number; city?: CityId | null }

export interface CourierHallView {
  of: PlayerId;
  holder: PlayerId;
  city: CityId;
  tile?: Hex;
}
export interface ImperialDiwanView { of: PlayerId; holder: PlayerId; city: CityId }
/** A city that carries a faction goal's VP for whoever holds it (core `view::GoalCityView`): Rome's Colonia Victrix, Egypt's Royal Tomb. */
export interface GoalCityView { of: PlayerId; holder: PlayerId; city: CityId; at: Hex; name: string }

export interface FactionGoalView {
  value: number;
  target: number;
  /** The measure in plain words, written by the core ("3 of 5 Ruhr Foundries"). */
  label: string;
  /** A Live goal scores only while it holds, and can be lost. */
  live: boolean;
  /** It scores now. */
  done: boolean;
}
/** Core `state::WonderLoss`: the race this city lost, its banked Production and the Gold refunded for it. */
export interface WonderLoss {
  turn: number;
  winner: PlayerId;
  building: BuildingKind;
  production: number;
  gold: number;
}
export interface GreatWorkView {
  player: PlayerId;
  completed: BuildingKind | null;
  building: BuildingKind | null;
  city: CityId | null;
  progress: number;
  cost: number;
  production_per_turn: number;
  turns: number | null;
  blocker: string | null;
}

/** Compact map payload. Detailed explanations are requested only for the inspected hex. */
export interface SettlementScore {
  pos: Hex;
  score: number;
  /** The site's absolute value and band (core `SiteBand`): the same for any Settler on any map. */
  quality: number;
  band: SiteBand;
  /** A visible foreign border makes an unseen city within spacing range possible. */
  border_risk: boolean;
  /** Rome only: this founder, founding here, founds the Colonia Victrix (core `goals::known_colonia_site`). Absent when false. */
  colonia?: boolean;
  rank: number | null;
  travel_turns: number | null;
}

export const SITE_BANDS = ['Poor', 'Fair', 'Good', 'Great', 'Exceptional'] as const;
export type SiteBand = typeof SITE_BANDS[number];

/** Why a tile is not a legal city site (core `SiteBlocker`, serde's external tagging). */
export type SiteBlocker =
  | 'Unexplored' | 'FailedBefore' | 'CityTooClose'
  | { Water: Terrain } | { Impassable: Terrain } | { Wonder: Wonder }
  | { Foreign: PlayerId }
  | { TooClose: { city: string; owner: PlayerId; distance: number } };

export interface SettlementSite extends SettlementScore {
  breakdown: { label: string; value: number }[];
  potential_yields: Yields;
  center_yields: Yields;
  /** Ring-one tiles a city founded here would own and work from the start (all of them, in hex order). */
  ring_one: { pos: Hex; terrain: Terrain; yields: Yields }[];
  terrain_bonus: Yields;
  defense_bonus: number;
  nearest_city: number | null;
  foreign_tiles: number;
  own_border_tiles: number;
  unknown_tiles: number;
  threats: number;
  /** Known resources and unowned wonders within the work radius that make up the quality. */
  resources: { resource: Resource; distance: number }[];
  wonders: { wonder: Wonder; distance: number; value: number }[];
}

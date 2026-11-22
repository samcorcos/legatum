// Editable test-only state projected from a Rust WASM save. Gameplay always runs in the core.
import type {
  BuildingKind,
  CityId,
  Faction,
  GameSetup,
  GoalRule,
  Hex,
  PlayerId,
  ProductionItem,
  Resource,
  TeamId,
  TechId,
  Terrain,
  Wonder,
  UnitId,
  UnitKind,
} from '../types';

export interface MUnit {
  id: UnitId;
  owner: PlayerId;
  kind: UnitKind;
  pos: Hex;
  hp: number;
  /** Moves left in the current resolution (reset at the start of each resolution). */
  moves: number;
  goto: Hex | null;
  path?: Hex[];
  path_stop_on_block?: boolean;
  automation?: import('../types').Automation | null;
  standing_order?: import('../types').StandingOrder | null;
  /** Stopped automation requests a new decision until ordered again. */
  needs_orders?: boolean;
  /** Consecutive turns without progress because the next step was blocked (core `blocked_turns`). */
  blocked_turns?: number;
  /** The unit an explicit melee attack order pursues (core `attack_target`). */
  attack_target?: UnitId;
  /** Bought Siege (core `siege`): this unit's attacks on a city ignore the Walls its owner's technology reaches. */
  siege?: boolean;
  /** Cumulative combat experience (core `xp`); the promotion level derives from it. */
  xp?: number;
  fortify: number;
  /** Moved or fought this turn (no healing, no fortify gain). */
  acted: boolean;
  /** Spawned, bought, upgraded or captured this turn: shows 0 moves until the next resolution. */
  fresh: boolean;
  /** Teams this unit attacked, during the one-turn visibility window. */
  attack_revealed_to?: TeamId[];
  /** Latest turn this attacker was revealed to its victim's team. */
  attack_revealed_turn?: number;
  /** An Heir only (core `Unit::heir_of`): the Byzantium empire whose goal he carries. A captured Heir keeps it. */
  heir_of?: PlayerId;
  /** A hired Mercenary Company (core `Unit::contract_until`, `origin`): the last turn of its contract and the faction that sold it. Fixtures may arrange these fields before loading them into the core. */
  contract_until?: number;
  origin?: Faction;
}

export interface MCity {
  id: CityId;
  owner: PlayerId;
  founder: PlayerId;
  name: string;
  pos: Hex;
  population: number;
  food: number;
  prod: number;
  producing: ProductionItem | null;
  productionQueue?: ProductionItem[];
  productionQueueNotified?: ProductionItem[];
  lastPurchaseTurn?: number | null;
  /** Funded production waits for completion; only its purchased portion is lost on a switch. */
  bought: { item: ProductionItem; cost: number; progress: number } | null;
  /** Progress banked per item when the city switched away (core `production_banked`), sorted by
   *  item order. `prod` is the active unit/building's progress, or unassigned Production when
   *  nothing (or a project) is active. Absent in old saves and when empty. */
  productionBanked?: { item: ProductionItem; progress: number }[];
  buildings: BuildingKind[];
  hp: number;
  /** The last turn the city took battle damage; it doesn't heal at the end of that turn (core `last_attacked_turn`). */
  lastAttackedTurn?: number;
  /** Tiles claimed by population growth, oldest first: one per population above 1 (core `grown_tiles`). */
  grownTiles: Hex[];
  tilesClaimed: number;
  isCapital: boolean;
  /** Stays true after the capital changes hands (core `City::original_capital`): what makes a captured city score the capital VP. */
  originalCapital?: boolean;
  foundedTurn: number;
  resistance: number;
}


export interface MPlayer {
  id: PlayerId;
  name: string;
  faction: Faction;
  team: TeamId;
  isHuman: boolean;
  alive: boolean;
  gold: number;
  techs: TechId[];
  research: TechId | null;
  research_queue: TechId[];
  /** Stored science; carries over when switching research. */
  science: number;
  settlersTrained: number;
  militaryPurchases?: number;
  /** Scored Age cards in a core save. */
  eras?: Record<number, { entered: number; scored?: { turn: number; slot: number } }>;
  greatWorkCompleted?: BuildingKind;
  greatWorkHolder?: PlayerId | null;
  factionGoalDone: boolean;
  /** The goal has scored at least once (a Live goal regained is not a first completion). */
  factionGoalReachedBefore?: boolean;
  /** Core `Player::goal_holder`: who scored this empire's held goal (the Heir's holder) at the last scoring. */
  goalHolder?: PlayerId | null;
  /** Core `Player::hall_city`: Mongols's Yam Relay Station city. The Station exists only while that city still stands with the building (`hallCity` in rules). */
  hallCity?: CityId | null;
  /** Core `Player::diwan_city`: India's Imperial Diwan city, including after capture. */
  diwanCity?: CityId | null;
  /** Core `Player::colonia_city`: Rome's Colonia Victrix, including after capture. */
  coloniaCity?: CityId | null;
  /** Core `Player::tomb_work`: Egypt is working on the Royal Tomb. */
  tombWork?: boolean;
  /** Core `Player::tomb_turns`: turns of work on the Royal Tomb so far. */
  tombTurns?: number;
  /** Core `Player::tomb_city`: the Royal Tomb's city, including after capture. */
  tombCity?: CityId | null;
  /** Free technologies a rival's broadcast left this empire to choose (core `free_techs_pending`). */
  freeTechsPending?: number;
  /** Whose broadcast granted them. */
  freeTechsFrom?: PlayerId | null;
  /** Sparta goal, Honour the Fallen: veterans retired so far (core `veterans_retired`). */
  veteransRetired?: number;
  /** Reserved Japan Sakoku vow (core `Player::sakoku_turns`): turn ends spent closed; absent or null while the country is open. */
  sakokuTurns?: number | null;
  /** A goal rule other than the faction's own (core `Player::goal_rule`; scenarios and tests). */
  goalRule?: GoalRule;
  /** Science generated in each of the last 5 turns (Most Advanced title). */
  scienceHistory: number[];
  /** 1 = explored by this player's own units and cities. Teams share (union) in views. */
  explored: number[];
  /** Scripted AI trait: hunts nearby enemies. */
  aggressive: boolean;
  start: Hex;
  citiesNamed: number;
  /** Strategic supply users (Forges, Factories, Pikemen), with since when each has used supply and whether it is idle (core `Player::supply`, `resources.ts updateSupply`). Absent in old saves. */
  supply?: { user: { kind: 'Building'; city: CityId; building: BuildingKind } | { kind: 'Unit'; unit: UnitId }; resource: Resource; since: number; idle?: boolean }[];
}

export interface MState {
  version: 1 | 2;
  /** Camps and raids (core `GameState::barbarians`, same JSON shape); absent when off. */
  barbarians?: { camps: { id: number; pos: Hex; founded_turn: number; guard: UnitId | null; scout: UnitId | null; raiders: UnitId[]; scout_report: CityId | null; raid: unknown; next_raid_turn: number; next_spawn_turn: number; rite?: PlayerId }[]; next_camp_id: number; last_camp_turn: number; known: Record<string, Hex[]> };
  diplomacy: import('../types').DiplomacyView & { contacts?: [number, number][] | null; next_proposal_id: number; next_war_id: number; next_treaty_id: number };
  setup: GameSetup;
  turn: number;
  width: number;
  height: number;
  terrain: Terrain[];
  wonder?: (Wonder | null)[];
  wondersDiscovered?: number[];
  /** 1 = unexplored ancient ruin. */
  ruin: number[];
  /** Map resource per tile (G11); absent in old saves, which then have none. */
  resource?: (Resource | null)[];
  /** 1 = the tile's resource is improved (core `Tile::improved`); absent in old saves. */
  improved?: number[];
  /** Built tile wonders, public to every empire. */
  landmark?: ({ building: BuildingKind; city: CityId; of: PlayerId } | null)[];
  /** Road links per tile (core `Tile::road`: bit d links direction d, 64 = road tile); absent until roads are laid (`roads.ts`). */
  road?: number[];
  /** Where the road network is rooted: the first Yam Relay Station's city tile (core `Map::road_hub`). */
  roadHub?: Hex;
  /** Whether any tile carries a road (core `Map::roaded`): Station roads or a city's Roadworks. */
  roaded?: boolean;
  /** Owner per tile, -1 = none. */
  owner: number[];
  /** Territory city per tile, -1 = none. */
  tileCity: number[];
  players: MPlayer[];
  units: MUnit[];
  cities: MCity[];
  nextUnitId: number;
  nextCityId: number;
  /** Last known owner per tile, per team (key = team id). -1 = none. */
  knownOwner: Record<string, number[]>;
  knownTerrain?: Record<string, number[]>;
  knownRuins?: Record<string, number[]>;
  /** Last seen improvement per tile, per team (core `TileMemory::improved`). */
  knownImproved?: Record<string, number[]>;
  /** Observations only: map reveals must not manufacture sightings of enemy cities. */
  knownCities?: Record<string, Record<number, MCity>>;
  winner: TeamId | null;
  finished: boolean;
  /** Core `GameState::vp_holds`: consecutive turn ends each team has held its VP target (present only while holding). */
  vpHolds?: Record<number, number>;
  /** Core `GameState::era_firsts`: the first empire into each era (era index to player); Ancient has none. */
  eraFirsts?: Record<number, PlayerId>;
}

export type MCamp = NonNullable<MState['barbarians']>['camps'][number];

/** Test-only editable saves. Every initial map and every resolved turn comes from Rust WASM. */
import { Game, initSync, explore_step } from '../wasm-pkg/legatum.js';
import { WasmClient } from '../wasmClient';
import { FACTIONS, FIRST_GROWTH, GROWTH_BASE, GROWTH_PER_POP, unitStatsFor } from '../data';
import { tileIndex } from '../hex';
import type { Faction, GameSetup, Hex, UnitKind, PlayerView, Order, TurnResult, OrderError, AttackMode, BattleForecast, CityAssaultForecast, AlongsideAttack, ProductionItem, CityYields, BuildingKind } from '../types';
import type { MCity, MPlayer, MState, MUnit } from './fixtureState';
import { coreFixture } from './coreFixture';

export const BARBARIAN = 255;
export const ROAD_NODE = 64;
export const IDLE_PRODUCTION_CAP = 60;
export const NOT_MET = "You haven't met this empire yet: offers need contact first (see one of their units, cities or borders)";
// Playwright also builds fixtures in Node. Browser fixtures use the WASM instance
// initialized by app startup; keep Node-only modules out of Vite's browser graph.
if (typeof window === 'undefined') {
  const { readFileSync } = await import('node:fs');
  const { resolve } = await import('node:path');
  initSync({ module: readFileSync(resolve(process.cwd(), 'src/wasm-pkg/legatum_bg.wasm')) });
}
const engine = () => Game;

export function makeStandardSetup(seed: string, f1: Faction, f2: Faction, n1: string, n2: string): GameSetup {
  const setup = JSON.parse(engine().standard_setup(seed, f1, f2, n1, n2)) as GameSetup;
  return setup;
}

/** Project an actual core save into the small editable fixture shape. */
export function stateFromCore(save: string): MState {
  const raw = JSON.parse(save) as Record<string, any>;
  const width: number = raw.map.width, height: number = raw.map.height;
  const tiles: any[] = raw.map.tiles;
  const units: MUnit[] = Object.values(raw.units).map((u: any) => ({
    ...u, moves: u.moves_left, fresh: false, path: u.path ?? [],
  }));
  const cities: MCity[] = Object.values(raw.cities).map((c: any) => ({
    ...c, food: c.food_stored, prod: c.production_stored, productionQueue: c.production_queue,
    productionBanked: c.production_banked, productionQueueNotified: c.production_queue_notified,
    lastPurchaseTurn: c.last_purchase_turn, lastAttackedTurn: c.last_attacked_turn,
    grownTiles: c.grown_tiles, tilesClaimed: c.tiles_claimed, isCapital: c.is_capital, originalCapital: c.original_capital,
    foundedTurn: c.founded_turn,
  }));
  const players: MPlayer[] = raw.players.map((p: any) => ({
    id: p.id, name: p.name, faction: p.faction, team: p.team, isHuman: p.is_human,
    alive: p.alive, gold: p.gold, science: p.science_stored, research: p.researching,
    research_queue: p.research_queue ?? [], techs: p.techs, settlersTrained: p.settlers_trained,
    militaryPurchases: p.military_purchases, eras: p.eras, factionGoalDone: p.faction_goal_done,
    factionGoalReachedBefore: p.faction_goal_reached_before, goalHolder: p.goal_holder,
    hallCity: p.hall_city, diwanCity: p.diwan_city, coloniaCity: p.colonia_city,
    tombWork: p.tomb_work, tombTurns: p.tomb_turns, tombCity: p.tomb_city, freeTechsPending: p.free_techs_pending,
    freeTechsFrom: p.free_techs_from, veteransRetired: p.veterans_retired,
    sakokuTurns: p.sakoku_turns, goalRule: p.goal_rule, scienceHistory: p.science_history ?? [],
    greatWorkCompleted: p.great_work_completed,
    explored: p.tile_memory.map((t: unknown) => t ? 1 : 0),
    aggressive: !p.is_human, start: cities.find((c) => c.owner === p.id && c.isCapital)?.pos ?? { q: 0, r: 0 },
    citiesNamed: 0,
    supply: p.supply?.map((r: any) => ({ ...r, user: r.user.kind === 'building'
      ? { kind: 'Building', city: r.user.city, building: r.user.building }
      : { kind: 'Unit', unit: r.user.unit } })),
  }));
  const knownOwner: Record<string, number[]> = {};
  const knownTerrain: Record<string, number[]> = {};
  const knownRuins: Record<string, number[]> = {};
  const knownImproved: Record<string, number[]> = {};
  for (const p of players) {
    const memory = raw.players[p.id].tile_memory as any[];
    knownOwner[p.team] = memory.map((t) => t?.owner ?? -1);
    knownTerrain[p.team] = memory.map((t) => t ? 1 : 0);
    knownRuins[p.team] = memory.map((t) => t?.ruin ? 1 : 0);
    knownImproved[p.team] = memory.map((t) => t?.improved ? 1 : 0);
  }
  return {
    version: 2, setup: raw.setup, turn: raw.turn, width, height,
    terrain: tiles.map((t) => t.terrain), ruin: tiles.map((t) => t.ruin ? 1 : 0),
    wonder: tiles.map((t) => t.wonder ?? null), resource: tiles.map((t) => t.resource ?? null),
    improved: tiles.map((t) => t.improved ? 1 : 0), road: tiles.map((t) => t.road ?? 0),
    roadHub: raw.map.road_hub, roaded: raw.map.roaded,
    owner: tiles.map((t) => t.owner ?? -1), tileCity: tiles.map((t) => t.city ?? -1),
    players, cities, units, nextUnitId: raw.next_unit_id, nextCityId: raw.next_city_id,
    knownOwner, knownTerrain, knownRuins, knownImproved,
    diplomacy: raw.diplomacy, barbarians: raw.barbarians,
    winner: raw.winner, finished: raw.finished, vpHolds: raw.vp_holds, eraFirsts: raw.era_firsts,
  };
}

export function createState(setup: GameSetup): MState {
  const game = new (engine())(JSON.stringify(setup));
  try { return stateFromCore(game.save()); } finally { game.free(); }
}

export const newBarbarians = (): NonNullable<MState['barbarians']> => ({ camps: [], next_camp_id: 0, last_camp_turn: 0, known: {} });
export function newDiplomacy(teams: number[]): MState['diplomacy'] {
  const setup = makeStandardSetup('1', 'Athens', 'India', 'A', 'B');
  setup.players = teams.map((team, id) => ({ ...setup.players[id % setup.players.length], name: `P${id}`, team }));
  return createState(setup).diplomacy;
}

export const idxOf = (s: MState, h: Hex): number => tileIndex(h, s.width, s.height);
export const growthThreshold = (city: MCity | number): number => {
  const pop = typeof city === 'number' ? city : city.population;
  return pop <= 1 ? FIRST_GROWTH : GROWTH_BASE + GROWTH_PER_POP * (pop - 1);
};
export function spawnUnit(s: MState, owner: number, kind: UnitKind, pos: Hex, fresh: boolean): MUnit {
  const stats = unitStatsFor(s.players[owner]?.faction ?? FACTIONS[0], kind);
  const u: MUnit = { id: ++s.nextUnitId, owner, kind, pos: { ...pos }, hp: 100,
    moves: fresh ? 0 : stats.moves, goto: null, fortify: 0, acted: false, fresh,
    ...(s.players[owner]?.faction === 'Sparta' && stats.military ? { xp: 16 } : {}) };
  s.units.push(u);
  return u;
}
export function meetAll(s: MState): void {
  const teams = [...new Set(s.players.map((p) => p.team))].sort((a, b) => a - b);
  if (s.diplomacy) s.diplomacy.contacts = teams.flatMap((a, i) => teams.slice(i + 1).map((b): [number, number] => [a, b]));
}

/** Resolve an arranged scene in Rust; callers may copy only the derived field they need. */
export function resolveFixture(s: MState, orders: Record<number, Order[]> = {}): MState {
  const client = new FixtureClient();
  client.load(coreFixture(s));
  client.endTurn(orders);
  return client.debugState();
}
export function viewFromFixture(s: MState, player: number): PlayerView {
  const client = new FixtureClient();
  client.load(coreFixture(s));
  return client.view(player);
}
export const buildView = viewFromFixture;
export function cityYields(s: MState, c: MCity, item?: ProductionItem): CityYields {
  const city = s.cities.find((x) => x.id === c.id);
  const prior = city?.producing;
  if (city && item) city.producing = item;
  try { return viewFromFixture(s, c.owner).cities.find((x) => x.id === c.id)!.detail!.yields; }
  finally { if (city && item) city.producing = prior!; }
}
export const productionCost = (s: MState, owner: number, item: ProductionItem): number =>
  engine().production_cost(coreFixture(s), owner, JSON.stringify(item));
export function buyCost(s: MState, c: MCity, item: ProductionItem | null = c.producing): number | null {
  if (!item) return null;
  const d = viewFromFixture(s, c.owner).cities.find((x) => x.id === c.id)!.detail!;
  return d.options.find((o) => JSON.stringify(o.item) === JSON.stringify(item))?.buy_cost ?? d.buy_cost;
}
export function canProduce(s: MState, c: MCity, item: ProductionItem): boolean {
  return viewFromFixture(s, c.owner).cities.find((x) => x.id === c.id)!.detail!.options
    .some((o) => JSON.stringify(o.item) === JSON.stringify(item));
}
export function buildingBlocker(s: MState, c: MCity, building: BuildingKind, _starting = true): string | null {
  const item = { kind: 'Building', building } as const;
  const view = viewFromFixture(s, c.owner);
  const d = view.cities.find((x) => x.id === c.id)!.detail!;
  return d.unavailable.find((o) => JSON.stringify(o.item) === JSON.stringify(item))?.reason
    ?? view.great_works?.find((g) => g.building === building && g.city === c.id)?.blocker ?? null;
}
export const swissInterest = (s: MState, owner: number): number =>
  viewFromFixture(s, owner).economy.gold_per_turn.parts.find(([label]) => label === 'Switzerland: interest')?.[1] ?? 0;
export const upkeepBreakdown = (s: MState, owner: number) => viewFromFixture(s, owner).economy.upkeep_breakdown!;
export const upgradeCost = (s: MState, u: MUnit): number | null =>
  viewFromFixture(s, u.owner).units.find((x) => x.id === u.id)?.upgrade_cost ?? null;
export const developmentForecast = (s: MState, owner: number) => viewFromFixture(s, owner).development!;
export const exploreStep = (view: PlayerView, unit: number): Order | null => JSON.parse(explore_step(JSON.stringify(view), unit));

/** Let the core lay Station and Roadworks links, preserving the arranged scene's other fields. */
export function refreshRoads(s: MState): void {
  const after = resolveFixture(s);
  s.road = after.road;
  s.roadHub = after.roadHub;
  s.roaded = after.roaded;
}

/** Ask core to derive visibility from the edited fixture, then remember only what it reveals. */
export function refreshVision(s: MState): void {
  const game = engine().load(coreFixture(s));
  try {
    for (const p of s.players) {
      const view = JSON.parse(game.view(p.id)) as PlayerView;
      const owner = s.knownOwner[p.team] ??= Array(s.terrain.length).fill(-1);
      const terrain = s.knownTerrain ??= {};
      const known = terrain[p.team] ??= Array(s.terrain.length).fill(0);
      const ruins = s.knownRuins ??= {};
      const knownRuin = ruins[p.team] ??= Array(s.terrain.length).fill(0);
      for (let i = 0; i < view.tiles.length; i++) if (view.tiles[i].visible) {
        p.explored[i] = 1;
        known[i] = 1;
        owner[i] = view.tiles[i].owner ?? -1;
        knownRuin[i] = view.tiles[i].ruin ? 1 : 0;
      }
    }
  } finally { game.free(); }
}

/** Native core client with a mutable fixture projection for tests that arrange a mid-game scene. */
export class FixtureClient extends WasmClient {
  private fixture: MState | null = null;
  private fixtureSnapshot: string | null = null;
  constructor() { super(engine() as ConstructorParameters<typeof WasmClient>[0]); }
  override async newGame(setup: GameSetup): Promise<void> { this.fixture = null; this.fixtureSnapshot = null; await super.newGame(setup); }
  override load(json: string): void {
    const raw = JSON.parse(json);
    this.fixture = Array.isArray(raw.terrain) ? raw as MState : null;
    super.load(this.fixture ? coreFixture(this.fixture) : json);
    this.fixtureSnapshot = this.fixture ? JSON.stringify(this.fixture) : null;
  }
  debugState(): MState {
    if (!this.fixture) {
      this.fixture = stateFromCore(super.save());
      this.fixtureSnapshot = JSON.stringify(this.fixture);
    }
    return this.fixture;
  }
  private sync() {
    if (!this.fixture) return;
    const current = JSON.stringify(this.fixture);
    if (current !== this.fixtureSnapshot) {
      super.load(coreFixture(this.fixture));
      this.fixtureSnapshot = current;
    }
  }
  override view(player: number): PlayerView { this.sync(); return super.view(player); }
  override validate(player: number, orders: Order[]): OrderError[] { this.sync(); return super.validate(player, orders); }
  override forecast(player: number, unit: number, target: Hex, mode: AttackMode): BattleForecast | null { this.sync(); return super.forecast(player, unit, target, mode); }
  override assault(player: number, unit: number, target: Hex, mode: AttackMode, alongside: AlongsideAttack[]): CityAssaultForecast | null { this.sync(); return super.assault(player, unit, target, mode, alongside); }
  override endTurn(orders: Record<number, Order[]>): TurnResult {
    this.sync();
    const players = this.fixture?.players ?? (JSON.parse(super.save()) as { players: { is_human: boolean }[] }).players.map((p, id) => ({ id, isHuman: p.is_human }));
    const humanOrders = Object.fromEntries(Object.entries(orders).filter(([id]) => players[Number(id)]?.isHuman));
    const result = super.endTurn(humanOrders);
    this.fixture = null;
    this.fixtureSnapshot = null;
    return result;
  }
  override save(): string { this.sync(); return super.save(); }
}

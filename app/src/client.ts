// The UI talks to the Rust core through WASM, or to the same core on the online host.
import type { TurnFrameTime } from './frameTelemetry';
import type {
  AttackMode,
  AlongsideAttack,
  BattleForecast,
  CityAssaultForecast,
  Faction,
  GameSetup,
  Hex,
  Order,
  OrderError,
  PlayerView,
  TurnResult,
} from './types';

import type { SettlementScore, SettlementSite, SiteBlocker } from './types';
import type { ScenarioStatus } from './scenarios/scenarios';

export interface GameClient {
  readonly kind: 'wasm' | 'remote';
  newGame(setup: GameSetup): Promise<void>;
  standardSetup(seed: string, faction1: Faction, faction2: Faction, name1: string, name2: string): Promise<GameSetup>;
  view(player: number): PlayerView;
  settlementSites?(view: PlayerView, unit: number, reserved: Hex[]): SettlementScore[];
  settlementSite?(view: PlayerView, unit: number, pos: Hex, reserved: Hex[]): SettlementSite | null;
  /** Why a hex is not a legal city site (core `site_blocker`), or null when it is one. */
  siteBlocker?(view: PlayerView, pos: Hex): SiteBlocker | null;
  validate(player: number, orders: Order[]): OrderError[];
  forecast(player: number, unit: number, target: Hex, mode: AttackMode): BattleForecast | null;
  /** The whole fight at an enemy city with your other queued attacks on it (core `forecast_city_assault`). */
  assault?(player: number, unit: number, target: Hex, mode: AttackMode, alongside: AlongsideAttack[]): CityAssaultForecast | null;
  /** Human orders keyed by seat; the client adds the AI orders and resolves the turn. */
  endTurn(humanOrders: Record<number, Order[]>): TurnResult;
  save(): string;
  /** Restore a game from save() output. May need the client to be initialized (see createClient). */
  load(json: string): void;
  /** Full owner history, available from the real local engine only. */
  archive?(): string;
  /** The same history with every seat under its default name, for upload. */
  publicArchive?(): string;
  recordTurnTime?(turn: number, endedAt: string, activeSeconds: number, era: number, sessionId: string): void;
  /** Attach a turn's browser frame and input timing (frameTelemetry.ts) to the archive. */
  recordFrameTime?(time: TurnFrameTime): void;
  forkArchive?(): void;
  /** Start a scenario from its file JSON (the real local engine only). */
  loadScenario?(scenarioJson: string): void;
  /** The scenario objective's status for the current state. */
  scenarioStatus?(): ScenarioStatus;
}

export type ClientKind = 'wasm';

export function requestedClientKind(search: string = globalThis.location?.search ?? ''): ClientKind | null {
  const v = new URLSearchParams(search).get('client');
  return v === 'wasm' ? v : null;
}

/** Create the local engine client. */
export async function createClient(_kind: ClientKind | null = requestedClientKind()): Promise<GameClient> {
  const { createWasmClient, wasmAvailable } = await import('./wasmClient');
  if (!wasmAvailable()) throw new Error('The game engine (WASM) is missing from this build. Run scripts/build-wasm.sh.');
  return createWasmClient();
}

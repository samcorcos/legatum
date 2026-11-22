// GameClient over the Rust core compiled to WASM (crates/wasm, story S8).
//
// Build the package into app/src/wasm-pkg with scripts/build-wasm.sh (-> app/src/wasm-pkg/legatum.js).
// Expected API (all JSON strings in and out):
//   new Game(setupJson)  Game.standard_setup(seed, f1, f2, n1, n2)  Game.load(json)
//   game.view(p)  game.validate(p, ordersJson)  game.forecast(p, unit, q, r, mode)
//   game.end_turn(humanOrdersJson) -> TurnResult JSON  game.save()  game.hash()  game.turn()
// save() is opaque (u64 bitset words): store the string as-is, never re-serialize it.
// forecast() returns BattleForecast JSON or the string "null". A Rust panic throws
// RuntimeError "unreachable" (real message on console.error); recreate from the last save.
import type { TurnFrameTime } from './frameTelemetry';
import type { GameClient } from './client';
import type { WhyCategory, WhyQuestion } from './why';
import type { TrainingEngine, TrainingFinish, TrainingProgress } from './training';
import type { SettlementScore, SettlementSite, SiteBlocker } from './types';
import type { ScenarioStatus, ScenarioTarget } from './scenarios/scenarios';
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

interface WasmGame {
  view(p: number): string;
  validate(p: number, ordersJson: string): string;
  forecast(p: number, unit: number, q: number, r: number, mode: AttackMode): string | null | undefined;
  assault(p: number, unit: number, q: number, r: number, mode: AttackMode, alongsideJson: string): string | null | undefined;
  end_turn(humanOrdersJson: string): string;
  save(): string;
  hash(): string;
  turn(): number;
  archive(): string;
  public_archive?(): string;
  record_turn_time?(turn: number, endedAt: string, activeSeconds: number, era: number, sessionId: string): void;
  record_frame_time?(timeJson: string): void;
  fork_archive(): void;
  attach_scenario(scenarioJson: string): void;
  scenario_status(): string;
  training_begin?(capMs: number): string;
  training_step?(maxRollouts: number, elapsedMs: number): string;
  training_finish?(elapsedMs: number): string;
  training_game_id?(): string;
  why_review?(player: number, ordersJson: string, askedJson: string): string;
  free?(): void;
}

interface WasmGameClass {
  new (setupJson: string): WasmGame;
  standard_setup(seed: string, f1: string, f2: string, n1: string, n2: string): string;
  load(json: string): WasmGame;
  load_scenario(scenarioJson: string): WasmGame;
  settlement_sites(view: string, unit: number, reserved: string): string;
  settlement_site(view: string, unit: number, q: number, r: number, reserved: string): string;
  site_blocker(view: string, q: number, r: number): string;
}

interface WasmModule {
  explore_step(view: string, unit: number): string;
  city_automation_orders(view: string, orders: string): string;
  scenario_targets?(scenarioJson: string): string;
  default?: (input?: unknown) => Promise<unknown>;
  Game: WasmGameClass;
}

// Vite resolves the optional package at build time. Playwright also imports
// fixture builders in Node, where import.meta.glob is unavailable.
const pkg: Record<string, () => Promise<WasmModule>> = import.meta.env
  ? import.meta.glob<WasmModule>('./wasm-pkg/legatum.js')
  : { './wasm-pkg/legatum.js': () => import('./wasm-pkg/legatum.js') as Promise<WasmModule> };

export function wasmAvailable(): boolean {
  return Object.keys(pkg).length > 0;
}

let modulePromise: Promise<WasmModule> | null = null;
let loadedModule: WasmModule | null = null;
/** Cheap synchronous previews may use the module after the game client has loaded it. */
export function loadedWasmModule(): WasmModule | null { return loadedModule; }
export function loadWasmModule(): Promise<WasmModule> {
  return modulePromise ??= (async () => {
    const loader = Object.values(pkg)[0];
    if (!loader) throw new Error('Auto-explore needs the WASM package. Run scripts/build-wasm.sh.');
    const mod = await loader();
    if (typeof mod.default === 'function') await mod.default();
    loadedModule = mod;
    return mod;
  })().catch((error) => { modulePromise = null; throw error; });
}

/** A scenario's objective cities from its start state, for the briefing; [] without the engine. */
export async function scenarioTargets(scenarioJson: string): Promise<ScenarioTarget[]> {
  if (!wasmAvailable()) return [];
  const mod = await loadWasmModule();
  return mod.scenario_targets ? JSON.parse(mod.scenario_targets(scenarioJson)) as ScenarioTarget[] : [];
}

export async function createWasmClient(): Promise<GameClient> {
  const loader = Object.values(pkg)[0];
  if (!loader) {
    throw new Error(
      'WASM client requested but app/src/wasm-pkg/legatum.js is missing. Build it with scripts/build-wasm.sh.',
    );
  }
  const mod = await loadWasmModule();
  return new WasmClient(mod.Game);
}

export class WasmClient implements GameClient, TrainingEngine {
  readonly kind = 'wasm' as const;
  private game: WasmGame | null = null;
  /** The scenario in play, re-attached when a save restores the game after an engine error. */
  private scenario: string | null = null;
  constructor(private readonly Game: WasmGameClass) {}

  private get g(): WasmGame {
    if (!this.game) throw new Error('No game in progress');
    return this.game;
  }

  async newGame(setup: GameSetup): Promise<void> {
    const replacement = new this.Game(JSON.stringify(setup));
    const previous = this.game;
    this.game = replacement;
    this.scenario = null;
    previous?.free?.();
  }

  async standardSetup(seed: string, f1: Faction, f2: Faction, n1: string, n2: string): Promise<GameSetup> {
    return JSON.parse(this.Game.standard_setup(seed, f1, f2, n1, n2));
  }

  view(player: number): PlayerView {
    return JSON.parse(this.g.view(player));
  }

  hash(): string { return this.g.hash(); }

  // Training mode (`training.ts`): the engine's battle search for this turn.
  trainingBegin(capMs: number): TrainingProgress | null {
    return this.g.training_begin ? JSON.parse(this.g.training_begin(capMs)) : null;
  }
  trainingStep(maxRollouts: number, elapsedMs: number): TrainingProgress | null {
    return JSON.parse(this.g.training_step!(maxRollouts, elapsedMs));
  }
  trainingFinish(elapsedMs: number): TrainingFinish | null {
    return JSON.parse(this.g.training_finish!(elapsedMs));
  }
  trainingGameId(): string { return this.g.training_game_id?.() ?? 'unknown'; }
  /** Coach questions: what Hard would have ordered in his seat, compared with his orders. */
  whyReview(player: number, orders: Order[], asked: { keys: string[]; last: [number, WhyCategory] | null; skipped?: WhyCategory[] }): { question: WhyQuestion | null; candidates: WhyQuestion[]; hard_orders: Order[] } {
    return JSON.parse(this.g.why_review!(player, JSON.stringify(orders), JSON.stringify(asked)));
  }

  settlementSites(view: PlayerView, unit: number, reserved: Hex[]): SettlementScore[] {
    return JSON.parse(this.Game.settlement_sites(JSON.stringify(view), unit, JSON.stringify(reserved)));
  }

  settlementSite(view: PlayerView, unit: number, pos: Hex, reserved: Hex[]): SettlementSite | null {
    return JSON.parse(this.Game.settlement_site(JSON.stringify(view), unit, pos.q, pos.r, JSON.stringify(reserved)));
  }

  siteBlocker(view: PlayerView, pos: Hex): SiteBlocker | null {
    return JSON.parse(this.Game.site_blocker(JSON.stringify(view), pos.q, pos.r));
  }

  validate(player: number, orders: Order[]): OrderError[] {
    return JSON.parse(this.g.validate(player, JSON.stringify(orders)));
  }

  forecast(player: number, unit: number, target: Hex, mode: AttackMode): BattleForecast | null {
    const s = this.g.forecast(player, unit, target.q, target.r, mode);
    if (!s || s === 'null') return null;
    return JSON.parse(s);
  }

  assault(player: number, unit: number, target: Hex, mode: AttackMode, alongside: AlongsideAttack[]): CityAssaultForecast | null {
    const s = this.g.assault(player, unit, target.q, target.r, mode, JSON.stringify(alongside));
    if (!s || s === 'null') return null;
    return JSON.parse(s);
  }

  endTurn(humanOrders: Record<number, Order[]>): TurnResult {
    return JSON.parse(this.g.end_turn(JSON.stringify(humanOrders)));
  }

  save(): string {
    return this.g.save();
  }

  archive(): string { return this.g.archive(); }
  publicArchive(): string { return this.g.public_archive ? this.g.public_archive() : this.g.archive(); }
  recordTurnTime(turn: number, endedAt: string, activeSeconds: number, era: number, sessionId: string) {
    this.g.record_turn_time?.(turn, endedAt, activeSeconds, era, sessionId);
  }
  recordFrameTime(time: TurnFrameTime) {
    this.g.record_frame_time?.(JSON.stringify(time));
  }
  forkArchive(): void { this.g.fork_archive(); }

  load(json: string): void {
    const replacement = this.Game.load(json);
    try {
      if (this.scenario) replacement.attach_scenario(this.scenario);
    } catch (error) {
      replacement.free?.();
      throw error;
    }
    const previous = this.game;
    this.game = replacement;
    previous?.free?.();
  }


  loadScenario(scenarioJson: string): void {
    const replacement = this.Game.load_scenario(scenarioJson);
    const previous = this.game;
    this.game = replacement;
    this.scenario = scenarioJson;
    previous?.free?.();
  }

  scenarioStatus(): ScenarioStatus {
    return JSON.parse(this.g.scenario_status());
  }
}

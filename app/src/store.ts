import { soundHooksForMoments } from './celebrate/feedback';
import { loadErrorMessage, olderRulesMessage, saveHasCurrentRules } from './saveErrors';
import { RULES } from './data';
import { buildLabel } from './buildLabel';
import { momentsForTurn, type Moment } from './celebrate/moments';
import { alarmsForTurn, type Alarm } from './celebrate/alarms';
import { hitsForTurn, type CityHit } from './siege';
// UI state: which seat is commanding, each seat's pending orders and Ready flag, selection,
// the last turn report and the unit-slide animation. The game itself lives in the GameClient.
import { create } from 'zustand';
import type { GameClient, ClientKind } from './client';
import { createClient } from './client';
import type {
  AirPick,
  BuildingKind,
  Event,
  Faction,
  GameSetup,
  Hex,
  Order,
  OrderError,
  PlayerView,
  TechId,
  TurnResult,
} from './types';
import { addAutomatedOrders, addCityAutomatedOrders, automatedUnits, hasAutomatedCities } from './automation';
import { AIR_ORDERS, nextUnitNeedingOrders } from './orders';
import { peacefulRangedAt } from './diplomacy';
import { nextDecision, pendingDecisions } from './turnFlow';
import { pendingRetire } from './goalActions';
import { seenGoalsAfterTurn } from './goalDecision';
import { spotsIn, strikeRange, type Hold } from './airWing';
import { releasedGarrison, type ReleasedUnit } from './releasedGarrison';
import { contextFromViews, formatEvents, movementPaths, objectiveCloseLines, releasedGarrisonLines, settlerSiteLines, type HoldCard, type ReportLine, type EventCategory } from './format';
import { buildCombatPlan, captureTime, type CombatPlan } from './map/combatFx';
import { warmAirArt } from './unitArt';
import { buildMomentPlan, movesEnd, type MomentPlan } from './map/momentFx';
import {
  RemoteClient,
  fetchStatus,
  serverBase,
  setServerBase,
  hostGame,
  rememberedSeat,
  type ConnState,
  type SeatStatus,
  type ServerStatus,
} from './remote';
import { StagingClient, type StagingState } from './staging';
import { rememberPlayerName, arrivedAsHost, inHostedWindow, markHostedWindow, online, returnToBundledApp, type OnlineInfo } from './platform/online';
import { HOST_AWAY, connectionFirst, roomBase } from './platform/relayLobby';
import { readNativeAutosave, writeNativeAutosave } from './platform/files';
import { checkSaveText } from './platform/saveLimits';
import { abandonGameHistory, recordGameHistory } from './platform/archive';
import { currentActivePlaySeconds, currentPlaySessionId, resetActivePlayTurn } from './activePlayTime';
import { cancelTurnFeedback, feel, feedbackForTurn, playTurnFeedback } from './feel';
import { newRun, saveRecord, type ScenarioEntry, type ScenarioFeedback, type ScenarioRun, type ScenarioStatus } from './scenarios/scenarios';
import { checkpointScenarioRun, saveScenarioRun, type SavedRun } from './platform/scenarioRuns';
import { NO_WHY, answerLine, remember as rememberWhy, writeAnswer, type WhyEngine, type WhyMemory, type WhyQuestion } from './why';
import { TrainingDriver, reportLine as trainingReportLine, trainingCapMs, writeTrainingLog, type TrainingEngine, type TrainingProgress, type TrainingUi } from './training';
import { inTauri } from './tauri';
import { setSeatFactions } from './theme';

function isMissingWasmError(message: string): boolean {
  return message.startsWith('The game engine (WASM) is missing from this build.');
}

type OrderFeedback = 'commit' | 'commitAttack' | 'queue' | 'purchase';

/** Index into the local human seats (`humans`); not a player id. */
export type Seat = number;
type Selection = { kind: 'unit' | 'air'; id: number } | { kind: 'city'; id: number } | null;
type Screen = 'title' | 'setup' | 'join' | 'staging' | 'game' | 'scenarios' | 'online';
/** What the setup screen is for: a single-player game, several humans on this screen, or hosting on the LAN. */
type SetupMode = 'solo' | 'hotseat' | 'host';

/** LAN play state (plans/lan-protocol.md). Null when playing locally (WASM hot-seat). */
export interface Lan {
  status: ServerStatus;
  /** Set once this browser has joined a seat. */
  mySeat: Seat | null;
  seats: SeatStatus[];
  conn: ConnState;
  /** The last join was refused; only an explicit loopback action may replace its token. */
  credentialRejected?: boolean;
}

/** A scenario in play (scenarios/*.json): its objective status and the run being recorded. */
export interface ActiveScenario {
  entry: ScenarioEntry;
  status: ScenarioStatus;
  run: ScenarioRun;
  /** The result screen is showing (it opens when the scenario ends). */
  resultOpen: boolean;
  /** Where the finished run was saved, or why it could not be. */
  saved: SavedRun | { error: string } | null;
  /** When the current turn began (ms since the epoch), for the time-per-turn record. */
  turnStartedAt: number;
}

export interface TurnReport {
  moments?: Moment[][];
  /** Per seat: big moments (a war declared on you, ...) shown full screen before the report. */
  alarms?: Alarm[][];
  turn: number;
  /** Lines per seat (each seat sees its own "Your ..." wording). */
  lines: ReportLine[][];
  /** Visible events per local seat; never the complete engine log. */
  events: Event[][];
  /** Per seat: the team's cities hit this turn, with HP before and after (`siege.ts`). */
  hits?: CityHit[][];
  /** Per seat: that seat's units the occupation lock released this turn; each asks for orders once. */
  released?: ReleasedUnit[][];
  hash: string;
}

interface MoveAnim {
  start: number;
  duration: number;
  paths: Map<number, Hex[]>;
  /** The turn's fighting, played back on the map when combat animations are on (`map/combatFx.ts`). */
  plan?: CombatPlan | null;
  /** The turn's payoff moments (founding, building, borders, captures), after the moves (`map/momentFx.ts`). */
  moments?: MomentPlan | null;
}

export const AUTOSAVE_KEY = 'legatum.autosave.v1';

// The opt-in app perf run attributes synchronous turn work without changing normal play.
const measureTurn = <T>(name: string, run: () => T): T => {
  if (typeof window === 'undefined' || new URLSearchParams(window.location.search).get('perf') !== '1') return run();
  const start = performance.now();
  try { return run(); }
  finally { performance.measure(`legatum:${name}`, { start, end: performance.now() }); }
};

export interface SaveFile {
  format: 'legatum-save';
  version: 1;
  client: ClientKind;
  savedAt: string;
  turn: number;
  names: string[];
  game: string;
  rulesVersion?: number;
  playtest?: string;
  /** Coach questions: the toggle and what was already asked (app-side; never part of the game state). */
  why?: WhyMemory;
}

interface State {
  screen: Screen;
  setupMode: SetupMode;
  client: GameClient | null;
  /** Player ids of the human seats played in this browser (one in solo and LAN, 2+ in hot-seat). */
  humans: number[];
  /** Per seat (index into `humans`). */
  views: PlayerView[] | null;
  seat: Seat;
  orders: Order[][];
  errors: OrderError[][];
  ready: boolean[];
  endTurnConfirmation: { seat: Seat; view: PlayerView; orders: Order[]; pass: boolean } | null;
  /**
   * Open "Declare War?" dialog: a military unit was ordered to attack (by moving, or `ranged`) a unit
   * or city of a team at Peace, or (`unit` null) War was chosen from a foreign city's card.
   */
  warPrompt: { seat: Seat; view: PlayerView; unit: number | null; target: Hex | null; team: number; ranged?: boolean; air?: boolean } | null;
  /** The own military unit selected just before a foreign city: its card offers "Attack with" it. */
  attacker: number | null;
  /**
   * G17: the confirmation shown over the map after a decision ("Alliance from Switzerland declined ·
   * Undo"). Undo restores `before` while the seat's orders are still exactly `after`; any other
   * order edit, a seat change or a new turn clears it.
   */
  notice: { id: number; text: string; seat: Seat; before: Order[]; after: Order[]; reopenTeam?: number } | null;
  selection: Selection;
  mode: 'normal' | 'ranged' | 'move';
  /** A finished technology whose unlock card was reopened from its turn-report line (null: none). */
  unlockReopen: TechId | null;
  /** Counts reopen requests, so asking for the same card twice shows it twice. */
  unlockReopenSeq: number;
  /** Your own era-up whose takeover was reopened from its turn-report line (null: none). Shares `unlockReopenSeq`. */
  eraReopen: { era: number; first: boolean } | null;
  /** The latest turn's report (map cues, celebrations and sounds read this one). */
  report: TurnReport | null;
  /** The last `REPORT_HISTORY` reports, oldest first, for this session; the latest is `report`. */
  reportHistory: TurnReport[];
  /** The turn the report view shows; null follows the latest. */
  reportTurn: number | null;
  /** A clicked combat line: the map draws an arrow from the attacker (when seen) to the target. */
  reportArrow: { from: Hex | null; to: Hex; seq: number } | null;
  reportOpen: boolean;
  reportFocus?: EventCategory;
  /** Seats that have dismissed the latest report's alarm screen (each local seat sees its own). */
  alarmSeen: Seat[];
  /** Number of this turn's priority-ordered interstitials already dismissed by each local seat. */
  alarmStep: Record<number, number>;
  /** Per player: the cities under attack the player has opened this turn. An opened alert stays in This Turn but stops being a pending decision. */
  siegeSeen: Record<number, { turn: number; cities: number[] }>;
  /** A victory-hold card reopened from its turn-report line (shown over the game until closed). */
  holdReopen: HoldCard | null;
  techOpen: boolean;
  /** The Cities overview (every city of yours in one table). */
  citiesOpen: boolean;
  /**
   * The city territory summary and the tile highlighted when the growth bar is hovered.
   */
  territory: { open: boolean; claimHover: boolean };
  /**
   * Tile improvements on the map: the selected city's eligible tiles are marked while its panel is
   * open; `hover` is the tile whose picker row is hovered or focused, `pick` the one a map click chose.
   */
  improve: { hover: Hex | null; pick: Hex | null };
  /**
   * Placing a tile-chosen build (the Yam Relay Station, a Great Work), as Civ places a district: the
   * city panel's Build, Queue or Buy opens it, the map lights every eligible tile, and a click on one
   * sets `pick`, which the panel turns into that order. Escape or a click elsewhere cancels. Buying a tile uses the same
   * mode (`intent: 'tile'`, no building): the map prices every tile the city can buy and a click on an affordable one queues it.
   */
  place: { city: number; building: BuildingKind | null; intent: 'build' | 'queue' | 'buy' | 'tile'; pick: Hex | null } | null;
  /**
   * The city panel's Air wing on the map: which aircraft has its strike or rebase picker open (its
   * candidates are ringed and a click on one picks it) and the candidate a row or the pointer is on.
   */
  air: { plane: number | null; what: AirPick | null; hover: Hex | null };
  /** Per player: the aircraft kept as they are this turn (a Fighter stays on guard, a Bomber holds), so they stop asking in the Return queue. */
  airHeld: Record<number, Hold>;
  diplomacyOpen: boolean;
  diplomacyTeam: number | null;
  /** Outcome dismissal belongs to the installed save/session, not the mounted game screen. */
  outcomeDismissed: boolean;
  /** Which part of the end of a finished game is showing: the staged reveal, then the post-game report. */
  outcomeStage: 'reveal' | 'report';
  /** Player ids whose elimination dialog has been dismissed in this session. */
  defeatDismissed: number[];
  /** Known elimination turns in this session; loaded saves may have no elimination history. */
  eliminationTurns: Record<number, number>;
  resolvingToEnd: boolean;
  vpExpanded: boolean;
  /** The Heir whose Disband is waiting for confirmation (it forfeits Byzantium's 2 VP). */
  heirDisbandAsk: number | null;
  /** The goal action being confirmed (the Great Migration or the broadcast): the dialog, and the map's preview of what it does. */
  goalDialog: 'Swarm' | 'Sakoku' | 'SakokuReopen' | 'Broadcast' | 'CallTheWild' | 'RetireVeteran' | null;
  /** The veteran whose retirement the confirm weighs (`goalDialog === 'RetireVeteran'`). */
  goalDialogUnit: number | null;
  /** Per player, the first ready faction-goal decision was opened or deferred. */
  goalDecisionSeen: Record<number, boolean>;
  /** The free-technology picker is open. */
  freeTechsOpen: boolean;
  /** The turn the free-technology prompt last opened for each seat's player: it reopens every turn until the choice is made. */
  freeTechsPrompted: Record<number, number>;
  anim: MoveAnim | null;
  /** Camera jump requests (the map watches this). `clear` also moves the tile out from under the panels (the report). */
  focus: { hex: Hex; seq: number; clear?: boolean } | null;
  busy: string | null;
  /** Training mode: this turn's battle search, while it runs (`training.ts`). */
  training: TrainingUi | null;
  /** Coach questions: the toggle and the record of what was asked (`why.ts`). */
  why: WhyMemory;
  /** The question waiting for his answer, if any. */
  whyQuestion: WhyQuestion | null;
  /** Esc tucked the waiting question's card away behind its "Coach question" button. */
  whyHidden: boolean;
  /** The last question he answered or skipped, so the turn report can list it (and reopen a skip). */
  whyRecent: { question: WhyQuestion; outcome: 'answered' | 'skipped' } | null;
  /** Answer (his words) or skip (null) the waiting question. */
  answerWhy(answer: string | null): void;
  /** Bring the coach's card back: the waiting question, or the last one he skipped if none is waiting. */
  showWhy(): void;
  error: string | null;
  /** Hot seat: the local seat whose player must take the device before its view is shown. */
  handoff: Seat | null;
  lastHash: string | null;
  /** Server mode: detected via GET /api/status at boot. */
  lan: Lan | null;
  /** Desktop app with online play: the relay and version to match; null elsewhere. */
  online: OnlineInfo | null;
  /** LAN staging room: the room as last broadcast, and this browser's seat in it. */
  staging: StagingState | null;
  stagingSeat: number | null;
  stagingConnected: boolean;
  stagingLeaving: boolean;
  booted: boolean;
  /** Bumped when an async forecast arrives so the map re-asks. */
  forecastTick: number;
  scenario: ActiveScenario | null;

  goto(screen: Screen): void;
  openSetup(mode: SetupMode): void;
  boot(): Promise<void>;
  refreshStatus(): Promise<ServerStatus | null>;
  /** LAN: create the game on the server (humans first; the host takes seat 0). */
  hostNewGame(setup: GameSetup): Promise<void>;
  joinSeat(seat: Seat, name: string): void;
  /** Online play: join a hosted game from the lobby list as a guest. Resolves to an error message, or null. */
  joinOnline(relay: string, room: string): Promise<string | null>;
  releaseLanSeat(seat: Seat): void;
  /** Take the seat back after another window took it over. */
  rejoinLan(): void;
  reclaimLanSeat(): void;
  leaveLan(): void;
  /** Back to the main menu; leaving a LAN game disconnects from it. */
  exitToTitle(): void;
  /** Reopen a finished game's report after it was closed to look at the map. */
  openGameReport(): void;
  /** Watch the staging room (host lobby and join screen). */
  enterStaging(host?: boolean): void;
  exitStaging(): void;
  takeStagingSeat(seat: number, name: string): void;
  releaseStagingSeat(seat: number): void;
  updateStagingSeat(p: { name?: string; faction?: Faction | 'Random' }): void;
  setStagingReady(ready: boolean): void;
  leaveStagingSeat(): void;
  startGame(setup: GameSetup, options?: { training?: boolean }): Promise<void>;
  loadSave(save: SaveFile, forkHistory?: boolean): Promise<void>;
  /** Start a scenario: its state with the seat as the only human, scored by the engine. */
  startScenario(entry: ScenarioEntry): Promise<void>;
  /** Attach the player's feedback to the finished run and save the run again. */
  submitScenarioFeedback(feedback: Omit<ScenarioFeedback, 'at'>): Promise<void>;
  setSeat(seat: Seat): void;
  select(sel: Selection, jump?: boolean): void;
  setMode(mode: 'normal' | 'ranged' | 'move'): void;
  editOrders(fn: (orders: Order[], view: PlayerView) => Order[], feedback?: OrderFeedback): void;
  nextUnit(): void;
  nextDecision(first?: boolean): void;
  /** Keep an aircraft as it is this turn: a Fighter stays on guard, a Bomber holds. It leaves the Return queue until next turn. */
  holdAir(plane: number): void;
  /** Put an aircraft to sleep: it asks nothing until an enemy is in reach. */
  sleepAir(plane: number): void;
  toggleReady(seat?: Seat): Promise<void>;
  resolveToEnd(): void;
  stopResolving(): void;
  jumpTo(hex: Hex, clear?: boolean): void;
  /** Show a finished technology's unlock card again (from the turn report). */
  reopenUnlock(tech: TechId): void;
  /** Show an era-up takeover again (from the turn report). */
  reopenEra(era: number, first: boolean): void;
  set(p: Partial<State>): void;
}

let focusSeq = 0;

function readViews(client: GameClient, humans: number[], firstView?: PlayerView): PlayerView[] {
  return humans.map((p) => p === firstView?.me ? firstView : client.view(p));
}

/** The human players of a game, from any player's view. */
function humanIds(view: PlayerView): number[] {
  const ids = view.players.filter((p) => p.is_human).map((p) => p.id);
  return ids.length ? ids : [0];
}

const emptySeats = (n: number) => ({
  endTurnConfirmation: null,
  warPrompt: null,
  attacker: null,
  notice: null,
  orders: Array.from({ length: n }, (): Order[] => []),
  errors: Array.from({ length: n }, (): OrderError[] => []),
  ready: Array.from({ length: n }, () => false),
});

/** Every seat whose empire is still alive is Ready (eliminated seats never hold up the turn). */
export function allReady(views: PlayerView[], ready: boolean[]): boolean {
  return views.every((v, i) => ready[i] || isEliminated(v));
}

export const isEliminated = (view: PlayerView): boolean =>
  view.players.find((p) => p.id === view.me)?.alive === false;

/** One connected LAN observer controls pacing; the others acknowledge its turns automatically. */
export function observerController(lan: Lan): SeatStatus | undefined {
  return lan.seats.filter((s) => s.connected).sort((a, b) => a.seat - b.seat)[0];
}

/** Observers may advance the AI game only after every human empire has fallen. */
export function canResolveToEnd(view: PlayerView, lan: Lan | null = null): boolean {
  if (view.finished || !isEliminated(view) || view.players.some((p) => p.is_human && p.alive)) return false;
  const controller = lan && observerController(lan);
  return !controller || controller.seat === lan?.mySeat;
}

export function saveFile(client: GameClient, views: PlayerView[]): SaveFile {
  return {
    format: 'legatum-save',
    version: 1,
    client: client.kind === 'remote' ? 'wasm' : client.kind,
    savedAt: new Date().toISOString(),
    turn: views[0].turn,
    names: views.map((v) => v.players.find((p) => p.id === v.me)?.name ?? `Player ${v.me + 1}`),
    game: client.save(),
    rulesVersion: RULES.saveVersion,
    playtest: buildLabel,
    ...(useGame.getState().why.enabled ? { why: useGame.getState().why } : {}),
  };
}

function parseAutosave(raw: string | null): SaveFile | null {
  try {
    if (!raw) return null;
    checkSaveText(raw);
    const s = JSON.parse(raw) as SaveFile;
    return s?.format === 'legatum-save' && s.version === 1 && s.client === 'wasm' &&
      typeof s.game === 'string' && Number.isFinite(Date.parse(s.savedAt)) && Number.isInteger(s.turn) &&
      Array.isArray(s.names) && s.names.every((name) => typeof name === 'string') ? s : null;
  } catch {
    return null;
  }
}

export async function readAutosave(): Promise<SaveFile | null> {
  // Wait for any queued disk write before comparing the two copies.
  const native = parseAutosave(await readNativeAutosave().catch(() => null));
  let local: SaveFile | null = null;
  try { local = parseAutosave(localStorage.getItem(AUTOSAVE_KEY)); } catch { /* Storage may be blocked. */ }
  return native && (!local || Date.parse(native.savedAt) > Date.parse(local.savedAt)) ? native : local;
}

function writeAutosave(client: GameClient, views: PlayerView[]) {
  if (client.archive && (views[0].finished || (views[0].turn - 1) % 5 === 0)) {
    try {
      void recordGameHistory(client.archive()).catch((error) => useGame.getState().set({ error: String(error) }));
    } catch (error) { useGame.getState().set({ error: `Could not record game history: ${String(error)}` }); }
  }
  try {
    const contents = JSON.stringify(saveFile(client, views));
    checkSaveText(contents);
    // The disk copy still runs when localStorage is full or unavailable.
    void writeNativeAutosave(contents).catch(() => {});
    localStorage.setItem(AUTOSAVE_KEY, contents);
  } catch {
    // Storage full or blocked: autosave is best-effort.
  }
}

function capitalOrUnit(v: PlayerView): Hex | null {
  const c = v.cities.find((c) => c.owner === v.me && c.is_capital) ?? v.cities.find((c) => c.owner === v.me);
  if (c) return c.pos;
  const u = v.units.find((u) => u.owner === v.me && u.kind === 'Settler') ?? v.units.find((u) => u.owner === v.me);
  return u?.pos ?? null;
}

export const useGame = create<State>((set, get) => {
  let pendingPurchase: { orders: string; changed: number[] } | null = null;
  const orderFeedback = (name: OrderFeedback, errors: OrderError[], changed: number[]) => {
    const error = errors.find((e) => e.index < 0 || changed.includes(e.index));
    if (error) {
      set({ error: error.message }); // Invalid feedback always has a visible reason.
      feel.invalid();
    } else feel[name]();
  };
  const validateSeat = (seat: Seat, orders: Order[]) => {
    const { client, humans } = get();
    if (!client) return [];
    try {
      if (client instanceof RemoteClient) client.syncOrders(orders);
      return client.validate(humans[seat], orders);
    } catch (e) {
      return [{ index: -1, message: String(e) }];
    }
  };

  // The last good save string, to recover after a WASM panic (which leaves the module unusable
  // until the game is recreated). Kept outside the store: it's large and never rendered.
  let lastGood: string | null = null;
  let observerTimer: ReturnType<typeof setTimeout> | null = null;
  const stopResolving = () => {
    if (observerTimer !== null) clearTimeout(observerTimer);
    observerTimer = null;
    set({ resolvingToEnd: false });
  };
  const queueObserverTurn = () => {
    if (!get().resolvingToEnd) return;
    if (observerTimer !== null) clearTimeout(observerTimer);
    // Yield between turns so Stop and the rest of the UI remain usable.
    observerTimer = setTimeout(() => {
      observerTimer = null;
      const s = get();
      const view = s.views?.[s.seat];
      if (!s.resolvingToEnd) return;
      if (!view || !canResolveToEnd(view, s.client?.kind === 'remote' ? s.lan : null) || s.screen !== 'game' || s.error ||
        (s.client?.kind === 'remote' && s.lan?.conn !== 'open')) {
        stopResolving();
        return;
      }
      if (!s.busy && !s.ready[s.seat]) s.toggleReady();
    }, 150);
  };
  const remember = (client: GameClient) => {
    try {
      lastGood = client.save();
    } catch {
      // keep the previous one
    }
  };

  const afterLoad = (client: GameClient) => {
    const previous = get().client;
    // A scenario's history is saved with its run, never in the game archive.
    if (previous !== client && previous?.archive && !get().scenario) {
      try {
        const history = previous.archive();
        const replacingSameGame = client.archive && JSON.parse(client.archive()).metadata.game_id === JSON.parse(history).metadata.game_id;
        if (!replacingSameGame) void abandonGameHistory(history).catch((error) => set({ error: String(error) }));
      }
      catch (error) { set({ error: String(error) }); }
    }
    stopResolving();
    // A local game replaces any LAN session: its socket must not reinstall itself later.
    dropRemote();
    remember(client);
    const firstView = client.view(0);
    const humans = humanIds(firstView);
    const views = readViews(client, humans, firstView);
    const hex = capitalOrUnit(views[0]);
    set({
      client,
      humans,
      views,
      goalDecisionSeen: {},
      outcomeDismissed: false, outcomeStage: 'reveal',
      handoff: null,
      defeatDismissed: [],
      eliminationTurns: {},
      screen: 'game',
      seat: 0,
      ...emptySeats(humans.length),
      selection: null,
      mode: 'normal',
      unlockReopen: null,
      eraReopen: null,
      report: null,
      reportHistory: [],
      reportTurn: null,
      reportArrow: null,
      lastHash: null,
      reportOpen: false,
      // Coach questions belong to one game: startGame and load set the toggle again after this.
      why: NO_WHY, whyQuestion: null, whyHidden: false, whyRecent: null,
      diplomacyOpen: false,
      diplomacyTeam: null,
      anim: null,
      busy: null,
      error: null,
      focus: hex ? { hex, seq: ++focusSeq } : null,
      scenario: null,
    });
    const first = nextUnitNeedingOrders(views[0], [], null);
    if (first) get().select({ kind: first.kind, id: first.id }, true);
    startTraining();
  };

  /** Training mode: the search behind this turn's AI orders, and its result for the report. */
  let trainingDriver: TrainingDriver | null = null;
  let trainedThisTurn: TrainingProgress | null = null;

  /** Start this turn's battle search while the human plans (games with a Training AI only). */
  const startTraining = () => {
    trainingDriver?.cancel();
    trainingDriver = null;
    trainedThisTurn = null;
    const client = get().client;
    const engine = client as unknown as Partial<TrainingEngine> | null;
    if (!client || client.kind !== 'wasm' || get().scenario || typeof engine?.trainingBegin !== 'function') {
      if (get().training) set({ training: null });
      return;
    }
    const capMs = trainingCapMs();
    let progress: TrainingProgress | null;
    try {
      progress = engine.trainingBegin(capMs);
    } catch (e) {
      console.error('legatum: training search could not start; the AI plays Hard this turn', e);
      progress = null;
    }
    if (!progress || progress.battles.length === 0) {
      if (get().training) set({ training: null });
      return;
    }
    const native = inTauri();
    const driver: TrainingDriver = new TrainingDriver(engine as TrainingEngine, progress, capMs, native, (p) => {
      const t = get().training;
      if (trainingDriver === driver && t) set({ training: { ...t, progress: p } });
    });
    trainingDriver = driver;
    set({ training: { progress, startedAt: driver.started, capMs, waiting: false, native } });
  };

  /** After a scenario turn: record it, and finish the run when the objective is decided. */
  const recordScenarioTurn = (client: GameClient, orders: Order[], result: TurnResult) => {
    const sc = get().scenario;
    if (!sc || !client.scenarioStatus) return;
    let status: ScenarioStatus;
    try {
      status = client.scenarioStatus();
    } catch (e) {
      set({ error: `Could not score the scenario: ${String(e)}` });
      return;
    }
    const me = get().views?.[0]?.me ?? -1;
    const seatLines = get().report?.lines[0] ?? [];
    const events = seatLines
      .filter((l) => l.weight > 0 && l.category !== 'movement')
      .map((l) => ({ text: l.text, tone: l.tone, category: l.category, weight: l.weight, mine: l.players.includes(me) }));
    const now = Date.now();
    const run: ScenarioRun = {
      ...sc.run,
      turns: [...sc.run.turns, { turn: result.turn, orders, hash: result.hash, score: status.score, outcome: status.outcome, turns_left: status.turns_left, ms: Math.max(0, now - sc.turnStartedAt), events }],
    };
    let archive: unknown = null;
    try { archive = client.archive ? JSON.parse(client.archive()) : null; } catch { /* the turns still hold every order */ }
    if (status.outcome === 'Playing') {
      set({ scenario: { ...sc, status, run, turnStartedAt: now } });
      // Kept as an unfinished run until it ends, in case the app closes mid-run (training data).
      void checkpointScenarioRun({ ...run, abandoned: true, finished_at: null, outcome: status, score: status.score, archive }).catch(() => { /* the next turn tries again */ });
      return;
    }
    const finished: ScenarioRun = { ...run, finished_at: new Date().toISOString(), outcome: status, score: status.score, archive };
    saveRecord(sc.entry.file.id, sc.entry.file.revision, status);
    set({ scenario: { ...sc, status, run: finished, resultOpen: true, saved: null, turnStartedAt: now }, reportOpen: false, techOpen: false, citiesOpen: false, selection: null });
    void saveScenarioRun(finished).then(
      (saved) => { if (get().scenario?.run === finished) set({ scenario: { ...get().scenario!, saved } }); },
      (e) => { if (get().scenario?.run === finished) set({ scenario: { ...get().scenario!, saved: { error: String(e instanceof Error ? e.message : e) } } }); },
    );
  };

  /** Leaving a scenario that is still being played keeps what was played (the coach reads it as "gave up"). */
  const saveAbandonedScenario = () => {
    const sc = get().scenario;
    if (!sc || sc.status.outcome !== 'Playing' || sc.run.turns.length === 0) return;
    const client = get().client;
    let archive: unknown = null;
    try { archive = client?.archive ? JSON.parse(client.archive()) : null; } catch { /* the turns still hold every order */ }
    const run: ScenarioRun = { ...sc.run, abandoned: true, finished_at: null, outcome: sc.status, score: sc.status.score, archive };
    set({ scenario: null });
    void saveScenarioRun(run).catch((e) => set({ error: `Your unfinished run could not be saved: ${String(e instanceof Error ? e.message : e)}` }));
  };

  /** New views after a resolution: report, slide animation, fresh orders. */
  const applyTurn = (prev: PlayerView[], next: PlayerView[], result: TurnResult) => {
    pendingPurchase = null;
    const seat = get().seat;
    const events = next.map((v) => get().client?.kind === 'remote'
      ? result.events : result.events_by_player?.[v.me] ?? []);
    const alarms = next.map((v, i) => alarmsForTurn(events[i], prev[i] ?? null, v));
    const lines = next.map((v, i) => {
      const ctx = contextFromViews(prev[i] ?? v, v);
      const trained = trainedThisTurn ? trainingReportLine(trainedThisTurn, v) : null;
      // A team two VP short of its target leads the report: it is the turn's biggest news.
      return [...alarms[i].filter((a) => a.kind === 'victoryNear').map((a) => a.line), ...formatEvents(events[i], ctx), ...releasedGarrisonLines(prev[i] ?? null, v, ctx), ...settlerSiteLines(prev[i] ?? null, v, ctx), ...objectiveCloseLines(prev[i] ?? null, v), ...(trained ? [trained] : [])];
    });
    trainedThisTurn = null;
    const paths = movementPaths(events[seat]);
    const steps = Math.max(0, ...[...paths.values()].map((p) => p.length - 1));
    const plan = buildCombatPlan(events[seat] ?? [], prev[seat] ?? null, next[seat]);
    if (plan) warmAirArt(plan.shots.flatMap((s) => s.kind === 'bombs' ? [s.craft ?? 'Bomber'] : s.kind === 'drone' ? ['drone' as const] : []));
    const glide = paths.size ? Math.min(1400, 260 + steps * 230) : 0;
    const moments = buildMomentPlan(events[seat] ?? [], prev[seat] ?? null, next[seat], movesEnd(plan, glide), plan ? (h) => captureTime(plan, h) : undefined);
    const team = next[seat].players.filter((p) => p.team === next[seat].team).map((p) => p.id);
    const eliminationTurns = { ...get().eliminationTurns };
    next.forEach((v, i) => {
      if (isEliminated(v) && !isEliminated(prev[i] ?? v)) eliminationTurns[v.me] = result.turn;
    });
    const report: TurnReport = { turn: result.turn, lines, events, hash: result.hash, moments: next.map((v, i) => momentsForTurn(events[i], v.me, prev[i] ?? null, v)),
      alarms,
      released: next.map((v, i) => releasedGarrison(prev[i] ?? null, v)),
      hits: next.map((v, i) => hitsForTurn(events[i], prev[i] ?? null, v)) };
    set({
      views: next,
      goalDecisionSeen: seenGoalsAfterTurn(prev, get().goalDecisionSeen),
      unlockReopen: null,
      eraReopen: null,
      alarmSeen: [],
      alarmStep: {},
      holdReopen: null,
      reportFocus: undefined,
      eliminationTurns,
      ...emptySeats(next.length),
      mode: 'normal',
      report,
      reportHistory: pushReport(get().reportHistory, report),
      reportTurn: null,
      reportArrow: null,
      // Open the report when something notable happened to this team.
      reportOpen: !isEliminated(next[seat]) && lines[seat].some((l) => l.weight > 0 && l.category !== 'movement' && l.players.some((p) => team.includes(p))),
      anim: paths.size || plan || moments ? { start: performance.now(), duration: glide, paths, plan, moments } : null,
      busy: null,
      // Order feedback describes the turn that just resolved; a new turn starts without it.
      error: null,
      lastHash: result.hash,
    });
    const client = get().client;
    playTurnFeedback(feedbackForTurn(events[seat], next[seat], prev[seat]), () => {
      const s = get();
      return s.screen === 'game' && s.client === client && s.seat === seat && s.views === next;
    }, soundHooksForMoments(get().report?.moments?.[seat] ?? [], events[seat]));
    const sel = get().selection;
    if (sel?.kind === 'unit' && !next[seat].units.some((u) => u.id === sel.id)) set({ selection: null });
    if (sel?.kind === 'air' && !next[seat].aircraft?.some((a) => a.id === sel.id)) set({ selection: null });
    if (next[seat].finished) stopResolving();
    else queueObserverTurn();
  };

  let remote: RemoteClient | null = null;

  /** Disconnect the LAN session; keep its credential available for a manual rejoin. */
  const dropRemote = () => {
    if (!remote) return;
    remote.disconnect(true);
    remote = null;
    const lan = get().lan;
    if (lan) set({ lan: { ...lan, mySeat: null, conn: 'closed', credentialRejected: false } });
  };
  let staging: StagingClient | null = null;

  /**
   * While a guest is cut off from a relayed game (the socket dropped or never opened), ask the room
   * whether it still exists. 404 means the game has ended: leave it. 503 means the host is away: say
   * so and keep waiting (the sockets reconnect by themselves). Nothing is retried forever on a 404.
   */
  let relayWatch: ReturnType<typeof setInterval> | null = null;
  const stopRelayWatch = () => {
    if (relayWatch) clearInterval(relayWatch);
    relayWatch = null;
  };
  const startRelayWatch = () => {
    stopRelayWatch();
    relayWatch = setInterval(async () => {
      const base = serverBase();
      if (!base) return stopRelayWatch();
      const s = get();
      const connected = s.stagingConnected || s.lan?.conn === 'open';
      if (connected) {
        if (s.error === HOST_AWAY) set({ error: null });
        return;
      }
      let code = 0;
      try {
        code = (await fetch(`${base}/api/status`, { cache: 'no-store', signal: AbortSignal.timeout(4000) })).status;
      } catch {
        return; // our own connection is down; the sockets keep retrying
      }
      if (serverBase() !== base) return;
      if (code === 404) {
        endRelaySession();
        set({ client: null, views: null, screen: 'title', busy: null, error: 'This game has ended.' });
      } else if (code === 503) set({ error: HOST_AWAY });
      else if (get().error === HOST_AWAY) set({ error: null });
    }, 4000);
  };

  /** A guest's room on the relay is over: forget it and go back to the app's own (same-origin) state. */
  const endRelaySession = () => {
    stopRelayWatch();
    if (!serverBase()) return;
    staging?.disconnect();
    staging = null;
    dropRemote();
    setServerBase('');
    set({ lan: null, staging: null, stagingSeat: null, stagingConnected: false, stagingLeaving: false });
  };

  const connectRemote = (seat: Seat, name: string, token?: string) => {
    stopResolving();
    remote?.disconnect(false);
    const rc = new RemoteClient(seat, name);
    if (token) rc.token = token;
    // An explicit new/load can create unclaimed seats. An old game's saved credential
    // must not prevent their initial claim; reserved seats always keep requiring it.
    else if (get().lan?.status.seats.find((s) => s.seat === seat)?.reserved === false) rc.token = null;
    remote = rc;
    const setLan = (p: Partial<Lan>) => {
      const lan = get().lan;
      if (lan) set({ lan: { ...lan, ...p } });
    };
    setLan({ credentialRejected: false });
    // Once another session replaces this one (a local game, another seat, leaving), a late
    // message or a reconnect from this socket must change nothing.
    const live = () => remote === rc;
    // Its seat-bound state only applies while it's the game on screen.
    const active = () => live() && get().client === rc;
    // The current LAN protocol still waits for connected seats, including eliminated ones.
    // Keep this observer Ready while living humans (or the first connected observer) set the pace.
    const readyObserver = () => {
      const view = get().views?.[0];
      if (!active() || rc.state !== 'open' || !view || view.finished || !isEliminated(view) ||
        canResolveToEnd(view, get().lan) || get().ready[0]) return;
      stopResolving();
      rc.syncOrders([]);
      if (rc.setReady(true)) set({ orders: [[]], errors: [[]], ready: [true] });
    };
    rc.on({
      connection: (conn) => {
        if (!live()) return;
        if (conn !== 'open') pendingPurchase = null;
        if (conn !== 'open') stopResolving();
        setLan({ conn, ...(conn === 'open' ? { credentialRejected: false } : {}) });
        if (conn === 'open') readyObserver();
      },
      // In LAN play this browser has exactly one local seat (index 0).
      seats: (seats) => {
        if (!live()) return;
        const me = seats.find((x) => x.seat === rc.seat);
        if (me && active()) set({ ready: [me.ready] });
        setLan({ seats });
        queueMicrotask(readyObserver);
      },
      validation: (errs, validatedOrders) => {
        if (!active()) return;
        set({ errors: [errs] });
        if (get().screen === 'game' && pendingPurchase && validatedOrders && pendingPurchase.orders === JSON.stringify(validatedOrders) &&
          pendingPurchase.orders === JSON.stringify(get().orders[0])) {
          const pending = pendingPurchase;
          pendingPurchase = null;
          orderFeedback('purchase', errs, pending.changed);
        }
      },
      forecast: () => active() && set({ forecastTick: get().forecastTick + 1 }),
      error: (message, code) => {
        if (!live()) return;
        stopResolving();
        if (code === 'invalid_seat_credential') {
          setLan({ credentialRejected: true });
          if (!active()) set({ screen: 'join' });
        }
        set({ error: message, busy: null });
      },
      view: (view, turn, resetOrders) => {
        if (!live()) return;
        set({ busy: null, error: null });
        const next = [view];
        const orders = [rc.pendingOrders()];
        const prev = get().views;
        // Run after RemoteClient has installed the view and completed its reconnect handshake.
        queueMicrotask(readyObserver);
        if (!prev || get().client !== rc) {
          // First view after joining.
          setLan({ mySeat: rc.seat });
          const hex = capitalOrUnit(view);
          set({
            client: rc,
            humans: [view.me],
            views: next,
            goalDecisionSeen: {},
            outcomeDismissed: false, outcomeStage: 'reveal',
            defeatDismissed: [],
            eliminationTurns: {},
            seat: 0,
            screen: 'game',
            ...emptySeats(1),
            orders,
            selection: null,
            report: null,
            reportHistory: [],
            reportTurn: null,
            reportArrow: null,
            lastHash: null,
            reportOpen: false,
            busy: null,
            error: null,
            focus: hex ? { hex, seq: ++focusSeq } : null,
          });
          const first = nextUnitNeedingOrders(view, orders[0], null);
          if (first) get().select({ kind: first.kind, id: first.id }, true);
          return;
        }
        if (turn) applyTurn(prev, next, turn);
        else if (resetOrders) {
          stopResolving();
          // A reconnect can skip the turn report, or return a different game at the same turn.
          set({
            views: next,
            outcomeDismissed: false, outcomeStage: 'reveal',
            defeatDismissed: [],
            eliminationTurns: {},
            ...emptySeats(1),
            orders,
            selection: null,
            mode: 'normal',
            report: null,
            reportHistory: [],
            reportTurn: null,
            reportArrow: null,
            reportOpen: false,
            anim: null,
            lastHash: null,
            busy: null,
          });
        } else set({ views: next, orders, errors: [[]], ready: [false] });
      },
    });
    rc.connect();
  };

  const resolveTurn = () => {
    const { client, orders, views, humans } = get();
    if (!client || !views) return;
    const endedAt = new Date().toISOString();
    const activeSeconds = currentActivePlaySeconds();
    set({ busy: 'Resolving turn…' });
    // Training mode: the AI's searched orders must be in before the turn resolves. The search
    // has run while he planned; the progress card covers whatever is left (at most the cap).
    const driver = trainingDriver;
    if (driver && !driver.done) {
      const t = get().training;
      if (t) set({ training: { ...t, waiting: true } });
      void driver.finish().then(() => {
        if (trainingDriver !== driver) return;
        trainedThisTurn = driver.result?.progress ?? null;
        setTimeout(resolveNow, 30);
      });
      return;
    }
    trainedThisTurn = driver?.result?.progress ?? null;
    // Let the overlay paint before the (synchronous) resolution.
    setTimeout(resolveNow, 30);

    function resolveNow() {
      if (!client || !views || get().client !== client) return;
      if (get().screen !== 'game') { set({ busy: null }); return; }
      let result: TurnResult;
      const submitted = humans.map((_, i) => isEliminated(views[i]) ? [] : orders[i]);
      // Coach questions: compare his orders with what Hard would have ordered in his seat.
      let asked: WhyQuestion | null = null;
      const why = get().why;
      const priorQuestion = get().whyQuestion;
      const engine = client as unknown as Partial<WhyEngine>;
      if (why.enabled && !get().scenario && !isEliminated(views[0]) && typeof engine.whyReview === 'function') {
        try {
          asked = engine.whyReview(humans[0], submitted[0], { keys: why.asked, last: why.last, skipped: priorQuestion ? [...(why.skipped ?? []), priorQuestion.category] : (why.skipped ?? []) }).question;
        } catch (e) {
          console.warn('legatum: training question check failed; the turn goes ahead', e);
        }
      }
      try {
        result = measureTurn('endTurn', () => client.endTurn(Object.fromEntries(humans.map((p, i) => [p, submitted[i]]))));
      } catch (e) {
        stopResolving();
        let recovered = false;
        if (lastGood) {
          try {
            client.load(lastGood);
            recovered = true;
          } catch {
            // fall through to the error below
          }
        }
        console.error('legatum: end turn failed', e);
        if (recovered) startTraining();
        set({
          busy: null,
          ready: humans.map(() => false),
          error: `The turn could not be resolved (${String(e)}).${recovered ? ' The game was restored to the start of this turn; your orders are kept.' : ''}`,
        });
        return;
      }
      if (asked) {
        // A question he never answered counts as skipped when the next one arrives.
        if (priorQuestion) get().answerWhy(null);
        set({ why: rememberWhy(get().why, asked), whyQuestion: asked, whyHidden: false });
      }
      if (result.training_lines?.length) {
        writeTrainingLog((client as unknown as TrainingEngine).trainingGameId(), result.training_lines);
      }
      if (!get().scenario && client.recordTurnTime) {
        try {
          const era = views[0].players.find((player) => player.id === views[0].me)?.era ?? 1;
          client.recordTurnTime(views[0].turn, endedAt, activeSeconds, era, currentPlaySessionId());
        } catch (e) {
          console.error('legatum: turn play time was not recorded', e);
        }
      }
      resetActivePlayTurn();
      const nextViews = measureTurn('readViews', () => readViews(client, humans));
      measureTurn('applyTurn', () => applyTurn(views, nextViews, result));
      // Scenarios never replace the campaign autosave; their runs are saved when they end.
      if (get().scenario) recordScenarioTurn(client, submitted[0], result);
      else measureTurn('writeAutosave', () => writeAutosave(client, get().views!));
      measureTurn('remember', () => remember(client));
      startTraining();
    }
  };

  return {
    screen: 'title',
    setupMode: 'solo',
    client: null,
    humans: [],
    views: null,
    seat: 0,
    ...emptySeats(0),
    selection: null,
    mode: 'normal',
    unlockReopen: null,
      eraReopen: null,
    unlockReopenSeq: 0,
    report: null,
    reportHistory: [],
    reportTurn: null,
    reportArrow: null,
    reportOpen: false,
    alarmSeen: [],
    alarmStep: {},
    siegeSeen: {},
    holdReopen: null,
    techOpen: false,
    citiesOpen: false,
    territory: { open: false, claimHover: false },
    improve: { hover: null, pick: null },
    place: null,
    air: { plane: null, what: null, hover: null },
    airHeld: {},
    diplomacyOpen: false,
    diplomacyTeam: null,
    outcomeDismissed: false, outcomeStage: 'reveal',
    defeatDismissed: [],
    eliminationTurns: {},
    resolvingToEnd: false,
    vpExpanded: false,
    heirDisbandAsk: null,
    goalDialog: null,
    goalDialogUnit: null,
    goalDecisionSeen: {},
    freeTechsOpen: false,
    freeTechsPrompted: {},
    anim: null,
    focus: null,
    busy: null,
    training: null,
    why: NO_WHY,
    whyQuestion: null,
    whyHidden: false,
    whyRecent: null,
    answerWhy: (answer) => {
      const q = get().whyQuestion;
      if (!q) return;
      const skipped = answer === null || !answer.trim();
      set({
        whyQuestion: null, whyHidden: false,
        whyRecent: { question: q, outcome: skipped ? 'skipped' : 'answered' },
        why: skipped ? { ...get().why, skipped: [...(get().why.skipped ?? []), q.category] } : get().why,
      });
      const client = get().client;
      const game = (client as unknown as Partial<WhyEngine> | null)?.trainingGameId?.() ?? 'unknown';
      writeAnswer(answerLine(game, q, answer));
    },
    showWhy: () => {
      // Reopened from the turn report: the report closes so the card never covers it.
      if (get().whyQuestion) { set({ whyHidden: false, reportOpen: false }); return; }
      const recent = get().whyRecent;
      if (recent?.outcome !== 'skipped') return;
      // Answering a skipped question later: the skip no longer counts against its category, and
      // the answer is appended as a second line for the same question (the later line wins).
      const skips = [...(get().why.skipped ?? [])];
      const at = skips.lastIndexOf(recent.question.category);
      if (at >= 0) skips.splice(at, 1);
      set({ whyQuestion: recent.question, whyHidden: false, reportOpen: false, whyRecent: null, why: { ...get().why, skipped: skips } });
    },
    error: null,
    handoff: null,
    lastHash: null,
    lan: null,
    online: null,
    staging: null,
    stagingSeat: null,
    stagingConnected: false,
    stagingLeaving: false,
    booted: false,
    forecastTick: 0,
    scenario: null,

    goto: (screen) => {
      if (screen !== 'game') stopResolving();
      if (screen === 'title') {
        // The host's window is the app's game server: its title screen is the bundled app's.
        if (inHostedWindow()) return returnToBundledApp();
        endRelaySession();
      }
      set({ screen });
    },
    openSetup: (setupMode) => set({ screen: 'setup', setupMode, error: null }),

    boot: async () => {
      if (get().booted) return;
      const status = await fetchStatus();
      // The desktop app's own page has no server; it may still offer online play.
      const onlineInfo = status ? null : await online().info();
      set({ booted: true, online: onlineInfo, lan: status ? { status, mySeat: null, seats: status.seats, conn: 'closed' } : null });
      // The host's window, opened by the app: go straight to the game room (unless a reload
      // resumes one below).
      const arrival = !!status?.is_local && arrivedAsHost();
      if (arrival) markHostedWindow();
      // A reloaded tab proves ownership with its session credential, even if status still
      // sees the old socket. Never bootstrap a released/replaced seat automatically.
      const remembered = rememberedSeat();
      if (remembered?.autoJoin === false) {
        if (arrival) set({ screen: 'setup', setupMode: 'host' });
        return;
      }
      if (remembered?.roomId && remembered.token && status) {
        const host = remembered.seat === 0 && status.is_local;
        set(host ? { screen: 'setup', setupMode: 'host' } : { screen: 'staging' });
        get().enterStaging(host);
        return;
      }
      const seat = remembered && status?.has_game ? status.seats.find((x) => x.seat === remembered.seat) : undefined;
      if (remembered?.token && seat && seat.reserved !== false) get().joinSeat(remembered.seat, remembered.name);
      else if (remembered && seat) set({ screen: arrival ? 'setup' : 'join', ...(arrival ? { setupMode: 'host' as const } : {}) });
      else if (arrival) set({ screen: 'setup', setupMode: 'host' });
    },

    joinOnline: async (relay, room) => {
      const previous = serverBase();
      setServerBase(roomBase(relay, room));
      const status = await fetchStatus(undefined, undefined, 6000);
      if (!status) {
        // Tell a room that is gone (404) from a host that is away (503) or an unreachable relay.
        let code = 0;
        try {
          code = (await fetch(`${serverBase()}/api/status`, { cache: 'no-store', signal: AbortSignal.timeout(4000) })).status;
        } catch {
          // offline
        }
        setServerBase(previous);
        return code === 404 ? 'This game has ended.' : code === 503 ? HOST_AWAY : 'Could not reach that game. Try again.';
      }
      startRelayWatch();
      set({ error: null, lan: { status, mySeat: null, seats: status.seats, conn: 'closed' }, screen: status.staging ? 'staging' : 'join' });
      return null;
    },

    refreshStatus: async () => {
      const status = await fetchStatus();
      const lan = get().lan;
      if (status) set({ lan: { ...lan, mySeat: lan?.mySeat ?? null, conn: lan?.conn ?? 'closed', status, seats: status.seats } });
      return status;
    },

    hostNewGame: async (setup) => {
      set({ busy: 'Surveying the map…', error: null });
      try {
        const status = await hostGame(setup);
        set({ busy: null, lan: { status, mySeat: null, seats: status.seats, conn: 'closed' } });
        get().joinSeat(0, setup.players[0].name);
      } catch (e) {
        set({ busy: null, error: `Could not host the game: ${String(e)}` });
      }
    },

    joinSeat: (seat, name) => {
      set({ busy: 'Joining…', error: null });
      connectRemote(seat, name);
    },

    releaseLanSeat: (seat) => {
      const lan = get().lan;
      const target = lan?.seats.find((s) => s.seat === seat);
      if (!lan?.status.is_local || seat === lan.mySeat || !target || target.connected || target.reserved === false) return;
      set({ error: null });
      if (!remote?.releaseSeat(seat)) set({ error: connectionFirst('open') });
    },

    rejoinLan: () => {
      set({ error: null });
      remote?.rejoin();
    },

    reclaimLanSeat: () => {
      const lan = get().lan;
      if (!lan?.status.is_local || !lan.credentialRejected || !remote || get().busy) return;
      set({ busy: 'Reclaiming…', error: null });
      remote.reclaimSeat();
    },

    leaveLan: () => {
      stopResolving();
      dropRemote();
      if (inHostedWindow()) return returnToBundledApp();
      endRelaySession();
      set({ client: null, views: null, screen: 'title', busy: null });
    },

    openGameReport: () => set({ outcomeDismissed: false, outcomeStage: 'report', reportOpen: false, techOpen: false, selection: null }),

    exitToTitle: () => {
      stopResolving();
      saveAbandonedScenario();
      // A join owns a socket before its first view installs the remote client.
      if (remote || get().client?.kind === 'remote') get().leaveLan();
      else get().goto('title');
    },

    enterStaging: (host = false) => {
      if (staging) return;
      staging = new StagingClient({
        connection: (connected) => set({ stagingConnected: connected }),
        leaving: (leaving) => set({ stagingLeaving: leaving }),
        left: () => set({ stagingSeat: null, error: null }),
        state: (state) => set((s) => ({
          staging: state, stagingSeat: staging?.seat ?? null,
          lan: s.lan ? { ...s.lan, status: { ...s.lan.status, staging: state } } : null,
        })),
        started: (seat, _gameId, token) => {
          const name = staging?.name ?? `Player ${seat + 1}`;
          staging = null;
          set({ staging: null, stagingSeat: null, busy: 'Joining…' });
          connectRemote(seat, name, token);
        },
        seatLost: (message) => set({ stagingSeat: null, error: message }),
        closed: () => set((s) => ({
          staging: null, stagingSeat: null, error: 'The host closed the game room.',
          lan: s.lan ? { ...s.lan, status: { ...s.lan.status, staging: null } } : null,
        })),
        error: (message) => set({ error: message, stagingSeat: staging?.seat ?? null }),
      }, undefined, undefined, host);
      staging.connect();
    },

    exitStaging: () => {
      staging?.disconnect();
      staging = null;
      set({ staging: null, stagingSeat: null, stagingConnected: false, stagingLeaving: false });
    },

    takeStagingSeat: (seat, name) => {
      rememberPlayerName(name);
      set({ error: null, stagingSeat: seat });
      staging?.take(seat, name);
    },

    releaseStagingSeat: (seat) => {
      const state = get();
      const occupant = state.staging?.occupants[seat];
      if (!state.lan?.status.is_local || seat === state.stagingSeat ||
        state.staging?.config.seats[seat]?.kind !== 'open' || !occupant || occupant.connected) return;
      set({ error: null });
      if (!staging?.release(seat)) set({ error: connectionFirst('open') });
    },

    updateStagingSeat: (p) => {
      if (p.name !== undefined) rememberPlayerName(p.name);
      staging?.update(p);
    },

    setStagingReady: (ready) => staging?.ready(ready),

    leaveStagingSeat: () => {
      if (get().stagingSeat === null || get().stagingLeaving) return;
      set({ error: null });
      if (!staging?.leave()) set({ error: connectionFirst('leave') });
    },

    set: (p) => set(p),

    startGame: async (setup, options) => {
      stopResolving();
      set({ busy: 'Surveying the map…', error: null });
      try {
        const client = await createClient();
        await client.newGame(setup);
        afterLoad(client);
        const first = get().views?.[0];
        const capital = first?.cities.find((city) => city.owner === first.me && city.is_capital);
        if (capital) get().select({ kind: 'city', id: capital.id }, true);
        if (options?.training) set({ why: { ...NO_WHY, enabled: true } });
        const views = get().views;
        if (views) writeAutosave(client, views);
      } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        set({ busy: null, error: isMissingWasmError(message) ? message : `Could not start the game: ${message}` });
      }
    },

    loadSave: async (save, forkHistory = false) => {
      stopResolving();
      set({ busy: 'Loading…', error: null });
      try {
        checkSaveText(save.game);
        if (!saveHasCurrentRules(save.game)) throw new Error(olderRulesMessage(save.playtest));
        if (save.client !== 'wasm') throw new Error('This save used the retired TypeScript engine and cannot be loaded by the Rust engine.');
        const client = await createClient(save.client);
        measureTurn('loadClient', () => client.load(save.game));
        if (forkHistory) client.forkArchive?.();
        measureTurn('afterLoad', () => afterLoad(client));
        if (save.why?.enabled) set({ why: { enabled: true, asked: save.why.asked ?? [], last: save.why.last ?? null, skipped: save.why.skipped ?? [] } });
        if (get().views) measureTurn('loadAutosave', () => writeAutosave(client, get().views!));
      } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        const refusal = isMissingWasmError(message) ? message : loadErrorMessage(e, save.playtest);
        set({ busy: null, error: get().screen === 'game' ? `${refusal} Your current game is unchanged.` : refusal });
      }
    },

    startScenario: async (entry) => {
      stopResolving();
      set({ busy: 'Loading…', error: null });
      try {
        const client = await createClient('wasm');
        if (!client.loadScenario || !client.scenarioStatus) throw new Error('scenarios need the WASM engine');
        client.loadScenario(entry.text);
        const status = client.scenarioStatus();
        afterLoad(client);
        // Open on the objective's city, else the seat's capital, rather than whichever unit
        // happens to need orders first.
        const view = get().views?.[0];
        const target = entry.file.objective.win?.map((c) => (c as { OwnCity?: number }).OwnCity).find((id) => id !== undefined);
        const home = view?.cities.find((c) => c.id === target) ?? view?.cities.find((c) => c.owner === view.me && c.is_capital) ??
          view?.cities.find((c) => c.owner === view.me);
        set({ scenario: { entry, status, run: newRun(entry, status, client.kind), resultOpen: false, saved: null, turnStartedAt: Date.now() } });
        if (home) {
          set({ selection: null });
          get().jumpTo(home.pos);
        }
      } catch (e) {
        set({ busy: null, error: `Could not start the scenario: ${String(e instanceof Error ? e.message : e)}` });
      }
    },

    submitScenarioFeedback: async (feedback) => {
      const sc = get().scenario;
      if (!sc || !sc.run.finished_at) return;
      const run: ScenarioRun = { ...sc.run, feedback: { ...feedback, at: new Date().toISOString() } };
      set({ scenario: { ...sc, run, saved: null } });
      try {
        const saved = await saveScenarioRun(run);
        if (get().scenario?.run === run) set({ scenario: { ...get().scenario!, saved } });
      } catch (e) {
        if (get().scenario?.run === run) set({ scenario: { ...get().scenario!, saved: { error: String(e instanceof Error ? e.message : e) } } });
      }
    },

    setSeat: (seat) => {
      if (get().client?.kind === 'remote') return; // one seat per browser in LAN play
      const views = get().views;
      if (!views || seat < 0 || seat >= views.length) return;
      set({ seat, selection: null, mode: 'normal', anim: null, endTurnConfirmation: null, warPrompt: null, attacker: null, notice: null });
      if (views) {
        const first = nextUnitNeedingOrders(views[seat], get().orders[seat], null);
        if (first) get().select({ kind: first.kind, id: first.id }, true);
        else {
          const h = capitalOrUnit(views[seat]);
          if (h) get().jumpTo(h);
        }
      }
    },

    select: (sel, jump = false) => {
      const { selection: prev, attacker, views, seat } = get();
      const v = views?.[seat];
      const foreignCity = sel?.kind === 'city' && v?.cities.some((c) => c.id === sel.id && v.players.find((p) => p.id === c.owner)?.team !== v.team);
      const fromUnit = prev?.kind === 'unit' && v?.units.some((u) => u.id === prev.id && u.owner === v.me && u.military) ? prev.id : null;
      // Clicking a foreign city with a unit selected keeps that unit at hand for "Attack with".
      const keep = foreignCity ? fromUnit ?? (prev?.kind === 'city' && prev.id === sel.id ? attacker : null) : null;
      if (sel?.kind === 'city' && v) {
        const seen = get().siegeSeen[v.me];
        const cities = seen?.turn === v.turn ? seen.cities : [];
        if (!cities.includes(sel.id)) set({ siegeSeen: { ...get().siegeSeen, [v.me]: { turn: v.turn, cities: [...cities, sel.id] } } });
      }
      set({ selection: sel, mode: 'normal', attacker: keep, territory: { ...get().territory, claimHover: false }, improve: { hover: null, pick: null }, place: null, air: { plane: null, what: null, hover: null } });
      if (jump && sel) {
        const v = get().views?.[get().seat];
        const pos = sel.kind === 'unit' ? v?.units.find((u) => u.id === sel.id)?.pos
          : sel.kind === 'air' ? v?.cities.find((c) => c.id === v?.aircraft?.find((a) => a.id === sel.id)?.city)?.pos
            : v?.cities.find((c) => c.id === sel.id)?.pos;
        if (pos) get().jumpTo(pos);
      }
    },

    setMode: (mode) => set({ mode }),

    editOrders: (fn, feedback) => {
      const { seat, views, orders, ready, errors } = get();
      if (!views || views[seat].finished || isEliminated(views[seat])) return;
      const next = fn(orders[seat], views[seat]);
      // Match the remote client's deduplication: no changed payload means the server stays Ready.
      if (JSON.stringify(next) === JSON.stringify(orders[seat])) return;
      pendingPurchase = null;
      const changed = next.flatMap((order, i) => orders[seat].some((old) => JSON.stringify(old) === JSON.stringify(order)) ? [] : [i]);
      const o = [...orders];
      o[seat] = next;
      const e = [...errors];
      e[seat] = validateSeat(seat, next);
      const r = [...ready];
      r[seat] = false; // editing orders un-readies the seat (the server does the same)
      // A strike or rebase is a new aircraft order: it wakes a plane waiting on guard.
      const held = get().airHeld;
      const owner = views[seat].me;
      const changedPlanes = changed.flatMap((i) => { const order = next[i]; return !e[seat].some((error) => error.index === i) && AIR_ORDERS.has(order.type) && 'plane' in order ? [order.plane] : []; });
      const oldHold = held[owner];
      const airHeld = changedPlanes.length && oldHold ? { ...held, [owner]: { ...oldHold,
        planes: oldHold.planes.filter((id) => !changedPlanes.includes(id)),
        asleep: oldHold.asleep?.filter((id) => !changedPlanes.includes(id)) } } : held;
      set({ orders: o, errors: e, ready: r, airHeld, endTurnConfirmation: null, notice: null });
      if (feedback) {
        if (feedback === 'purchase' && get().client?.kind === 'remote') {
          pendingPurchase = { orders: JSON.stringify(next), changed };
        } else orderFeedback(feedback, e[seat], changed);
      }
    },

    nextUnit: () => {
      const { views, seat, orders, selection, errors } = get();
      if (!views) return;
      const cur = selection?.kind === 'unit' || selection?.kind === 'air' ? selection.id : null;
      const valid = orders[seat].filter((_, i) => !errors[seat].some((e) => e.index === i));
      const u = nextUnitNeedingOrders(views[seat], valid, cur, get().airHeld[views[seat].me]);
      if (u) get().select({ kind: u.kind, id: u.id }, true);
      else set({ selection: null });
    },

    holdAir: (plane) => {
      const { views, seat, airHeld } = get();
      const v = views?.[seat];
      if (!v) return;
      const held = airHeld[v.me]?.turn === v.turn ? airHeld[v.me].planes : [];
      if (held.includes(plane)) return;
      set({ airHeld: { ...airHeld, [v.me]: { turn: v.turn, planes: [...held, plane], asleep: airHeld[v.me]?.asleep } }, air: { plane: null, what: null, hover: null } });
    },

    sleepAir: (plane) => {
      const { views, seat, airHeld } = get();
      const v = views?.[seat];
      if (!v) return;
      const now = airHeld[v.me];
      const asleep = now?.asleep ?? [];
      set({ airHeld: { ...airHeld, [v.me]: { turn: now?.turn ?? v.turn, planes: now?.planes ?? [], asleep: asleep.includes(plane) ? asleep.filter((id) => id !== plane) : [...asleep, plane] } }, air: { plane: null, what: null, hover: null } });
    },

    nextDecision: (first = false) => {
      const s = get();
      const view = s.views?.[s.seat];
      if (!view || s.busy || s.ready[s.seat]) return;
      const list = pendingDecisions(view, s.orders[s.seat], s.errors[s.seat], s);
      const decision = nextDecision(list, first ? null : s.selection);
      if (!decision) return;
      if (decision.kind === 'diplomacy') set({ selection: null, diplomacyOpen: true, diplomacyTeam: decision.team, reportOpen: false, techOpen: false, mode: 'normal' });
      else if (decision.kind === 'research') set({ selection: null, techOpen: true, mode: 'normal' });
      else if (decision.kind === 'freeTechs') set({ selection: null, freeTechsOpen: true, mode: 'normal' });
      else if (decision.kind === 'goal') {
        set({ goalDecisionSeen: { ...s.goalDecisionSeen, [view.me]: true } });
        if (decision.action === 'Retire') {
          const veteran = view.actions?.retire?.veterans.find((v) => !v.blocker && !pendingRetire(s.orders[s.seat], v.unit));
          if (veteran) set({ goalDialog: 'RetireVeteran', goalDialogUnit: veteran.unit });
        } else set({ goalDialog: decision.action });
      }
      else if (decision.kind === 'siege') get().select({ kind: 'city', id: decision.id }, true);
      else if (decision.kind === 'air') {
        get().select({ kind: 'air', id: decision.id }, true);
        // The picker opens only with something to pick; otherwise the card alone.
        const plane = view.aircraft?.find((a) => a.id === decision.id);
        const city = view.cities.find((c) => c.id === decision.city);
        const targets = plane && city ? spotsIn(view, city, strikeRange(plane.kind), s.orders[s.seat]).hostile.length : 0;
        set({ air: { plane: decision.id, what: targets ? 'strike' : null, hover: null } });
      }
      else if (decision.kind === 'treasury') { if (decision.target) get().select(decision.target, true); else set({ selection: null }); }
      else {
        get().select({ kind: decision.kind, id: decision.id }, true);
        if (decision.kind === 'unit') {
          const invalidRanged = s.orders[s.seat].find((o, i) => o.type === 'RangedAttack' && o.unit === decision.id
            && s.errors[s.seat].some((e) => e.index === i));
          const peace = invalidRanged?.type === 'RangedAttack' ? peacefulRangedAt(view, invalidRanged.target) : null;
          // An attack on a team at Peace is fixed by deciding on War: ask, as when ordering it.
          if (peace && invalidRanged?.type === 'RangedAttack') set({ warPrompt: { seat: s.seat, view, unit: decision.id, target: invalidRanged.target, team: peace.team, ranged: true } });
          else if (invalidRanged) set({ mode: 'ranged', error: decision.label });
        }
      }
      feel.select();
    },

    toggleReady: async (which) => {
      const { seat, ready, errors, orders, busy, views, scenario } = get();
      if (busy || !views || views[0].finished) return;
      // A decided scenario plays no further turns.
      if (scenario && scenario.status.outcome !== 'Playing') return;
      const s = which ?? seat;
      const observer = isEliminated(views[s]);
      if (observer && !canResolveToEnd(views[s], get().client?.kind === 'remote' ? get().lan : null)) return;
      const client = get().client;
      let submitted = orders;
      if (!ready[s] && !observer && (automatedUnits(views[s], orders[s]).length || hasAutomatedCities(views[s], orders[s]))) {
        set({ busy: 'Planning automation…' });
        try {
          const plan = await addCityAutomatedOrders(views[s], await addAutomatedOrders(views[s], orders[s]));
          // A load, disconnect, or order edit while WASM initializes invalidates this plan.
          if (get().client !== client || get().views !== views || get().orders !== orders || get().screen !== 'game') return;
          submitted = orders.map((list, i) => i === s ? plan.orders : list);
          set({ orders: submitted });
          if (plan.unchosen_cities.length) {
            // The same loaded WASM chooser now exposes these cities to pendingDecisions.
            set({ selection: { kind: 'city', id: plan.unchosen_cities[0] } });
            return;
          }
        } catch (error) {
          if (get().client === client) set({ error: `Could not plan automation: ${String(error)}` });
          return;
        } finally {
          if (get().client === client && get().busy === 'Planning automation…') set({ busy: null });
        }
      }
      const r = [...ready];
      if (remote && get().client === remote) {
        // LAN: the server resolves the turn once every human seat is Ready.
        if (s !== 0 || remote.state !== 'open') return;
        r[s] = !r[s];
        remote.syncOrders(observer ? [] : submitted[s]);
        if (!remote.setReady(r[s])) return;
        set({ ready: r, ...(observer ? { orders: [[]], errors: [[]] } : {}) });
        return;
      }
      r[s] = !r[s];
      const e = [...errors];
      if (r[s]) e[s] = observer ? [] : validateSeat(s, submitted[s]);
      set({ ready: r, errors: e });
      if (allReady(views, r)) resolveTurn();
    },

    resolveToEnd: () => {
      const s = get();
      const view = s.views?.[s.seat];
      if (!view || !canResolveToEnd(view, s.client?.kind === 'remote' ? s.lan : null) || s.busy || s.screen !== 'game' ||
        (s.client?.kind === 'remote' && s.lan?.conn !== 'open')) return;
      set({ resolvingToEnd: true, reportOpen: false, techOpen: false, citiesOpen: false, error: null,
        defeatDismissed: [...new Set([...s.defeatDismissed, view.me])] });
      queueObserverTurn();
    },
    stopResolving,

    jumpTo: (hex, clear) => set({ focus: { hex, seq: ++focusSeq, ...(clear ? { clear } : {}) } }),

    reopenUnlock: (tech) => set({ unlockReopen: tech, eraReopen: null, unlockReopenSeq: get().unlockReopenSeq + 1 }),
    reopenEra: (era, first) => set({ eraReopen: { era, first }, unlockReopen: null, unlockReopenSeq: get().unlockReopenSeq + 1 }),

  };
});

/** How many turn reports the report view can step back through (kept in memory for the session). */
const REPORT_HISTORY = 20;

/** Add a resolved turn's report; a replayed turn replaces its old entry; keep the last `max`. */
export function pushReport(history: TurnReport[], report: TurnReport, max = REPORT_HISTORY): TurnReport[] {
  return [...history.filter((r) => r.turn < report.turn), report].slice(-max);
}

/** The report the report view shows: the chosen turn, else the latest. */
export const viewedReport = (s: { report: TurnReport | null; reportHistory: TurnReport[]; reportTurn: number | null }) =>
  (s.reportTurn === null ? null : s.reportHistory.find((r) => r.turn === s.reportTurn)) ?? s.report;

/** Step the report view one turn back (-1) or forward (+1) through the history. */
export function stepReport(delta: -1 | 1): void {
  const s = useGame.getState();
  const shown = viewedReport(s);
  if (!shown) return;
  const i = s.reportHistory.findIndex((r) => r.turn === shown.turn);
  const next = s.reportHistory[i + delta];
  if (i < 0 || !next) return;
  s.set({ reportTurn: next === s.report ? null : next.turn, reportFocus: undefined, reportArrow: null });
}

/** The "Last turn" control and the L key: open the latest report, or close it if it's showing. */
export function toggleLastTurnReport(): void {
  const s = useGame.getState();
  if (!s.report) return;
  if (s.reportOpen) s.set({ reportOpen: false, reportArrow: null });
  else s.set({ reportOpen: true, reportTurn: null, reportFocus: undefined });
}

const NO_ALARMS: Alarm[] = [];
const ALARM_TAILS = new WeakMap<Alarm[], Map<number, Alarm[]>>();
/** The alarm screen this seat has yet to see: only for the latest turn, after any hot-seat hand-off. */
export const pendingAlarms = (s: Pick<State, 'report' | 'seat' | 'alarmSeen' | 'alarmStep' | 'handoff' | 'views'>): Alarm[] => {
  const alarms = s.report?.alarms?.[s.seat];
  if (!alarms?.length || s.alarmSeen.includes(s.seat) || s.handoff !== null || s.views?.[s.seat]?.finished) return NO_ALARMS;
  const step = s.alarmStep[s.seat] ?? 0;
  if (!step) return alarms;
  let tails = ALARM_TAILS.get(alarms);
  if (!tails) { tails = new Map(); ALARM_TAILS.set(alarms, tails); }
  let tail = tails.get(step);
  if (!tail) { tail = alarms.slice(step); tails.set(step, tail); }
  return tail;
};

// Empire colours follow the faction: tell the colour helpers who is who before any component renders the new views.
useGame.subscribe((state, previous) => {
  const players = state.views?.[0]?.players;
  if (players && state.views !== previous.views) setSeatFactions(players.map((p) => p.faction));
});

useGame.subscribe((state, previous) => {
  if (state.screen !== previous.screen || state.client !== previous.client || state.seat !== previous.seat) cancelTurnFeedback();
});

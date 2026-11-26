import { soundHooksForMoments } from './celebrate/feedback';
import { DAMAGED_SAVE_MESSAGE, loadErrorMessage, olderRulesMessage, saveHasCurrentRules, saveRulesVersion } from './saveErrors';
import { BUILD_ID, buildForSave, buildUrl, fetchVersions, handOffSave, navigate, pageLaunch } from './platform/versions';
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
  PlayerId,
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
import { coloniaFounder, seenGoalsAfterTurn } from './goalDecision';
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
  setJoinCode,
  hostGame,
  hostingHere,
  rememberSeat,
  rememberedSeat,
  rememberRoom,
  rememberedRoom,
  type ConnState,
  type SeatStatus,
  type ServerStatus,
} from './remote';
import { StagingClient, type StagingState } from './staging';
import { rememberPlayerName, online, type OnlineInfo } from './platform/online';
import { hostedGameSave, hostingRecord, startHosting, stopHosting } from './net/hosting';
import { RelayRefusal } from './net/browserHost';
import { HOST_AWAY, HOST_OFFLINE, connectionFirst, roomBase } from './platform/relayLobby';
import { checkSaveText } from './platform/saveLimits';
import { listAutosaves, newCampaignId, readPendingOrders, writeAutosave as keepAutosave, writePendingOrders, type AutosaveRecord } from './platform/autosaveStore';
import { abandonGameHistory, recordGameHistory } from './platform/archive';
import { historyToRecord, isHistoryCheckpoint } from './platform/gameUpload';
import { currentActivePlaySeconds, currentPlaySessionId, resetActivePlayTurn } from './activePlayTime';
import { takeTurnFrameTime } from './frameTelemetry';
import { mapViewport } from './map/camera';
import { cancelTurnFeedback, feel, feedbackForTurn, playTurnFeedback } from './feel';
import { NO_WHY, answerLine, remember as rememberWhy, MAX_SAVED_ANSWERS, withAnswer, writeAnswer, type WhyEngine, type WhyMemory, type WhyQuestion } from './why';
import { TrainingDriver, reportLine as trainingReportLine, trainingCapMs, writeTrainingLog, type TrainingEngine, type TrainingProgress, type TrainingUi } from './training';
import { setSeatFactions } from './theme';
import { PING_MIN_GAP_MS, pingAvailable, withPing, type TeamPing } from './teamPing';

function isMissingWasmError(message: string): boolean {
  return message.startsWith('The game engine (WASM) is missing from this build.');
}

type OrderFeedback = 'commit' | 'commitAttack' | 'queue' | 'purchase';

/** Index into this browser's human views (`humans`); not a player id. */
export type Seat = number;
type Selection = { kind: 'unit' | 'air'; id: number } | { kind: 'city'; id: number } | null;
type Screen = 'title' | 'setup' | 'join' | 'staging' | 'game' | 'online';
/** What the setup screen is for: a solo game or hosting an online game. */
type SetupMode = 'solo' | 'host';

/** Online play state, hosting or joined. Null in a solo game. */
export interface NetState {
  status: ServerStatus;
  /** Set once this browser has joined a seat. */
  mySeat: Seat | null;
  seats: SeatStatus[];
  conn: ConnState;
  /** The last join was refused; only an explicit loopback action may replace its token. */
  credentialRejected?: boolean;
  /** A guest: the host wants a join code for a seat (the lobby row said Locked, or the host refused a claim). */
  locked?: boolean;
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
  if (!(!(!(typeof window === 'undefined') && !(new URLSearchParams(window.location.search).get('perf') !== '1')))) {} else { return run(); }
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
  /** The website build that wrote it (platform/versions.ts): an older save opens there. Absent in older saves. */
  build?: string;
  /** Written by an online host's "Save to file": the game has human seats, and can be hosted again. */
  online?: true;
  /** Coach questions: the toggle and what was already asked (app-side; never part of the game state). */
  why?: WhyMemory;
  /** Every time the player went back to an earlier turn ("Load an earlier turn"), oldest first. Absent when never. */
  rewinds?: Rewind[];
}

/** Old local saves with several human empires must be resumed through an online room. */
export function needsOnlineHost(save: Pick<SaveFile, 'game'>): boolean {
  try {
    const game = JSON.parse(save.game) as { players?: { is_human?: boolean }[] };
    return (game.players?.filter((player) => player.is_human).length ?? 0) > 1;
  } catch { return false; } // The normal save parser reports malformed files.
}

/**
 * One "Load an earlier turn": the player left turn `from_turn` and played on from the start of `to_turn`. The uploaded
 * record carries the list as the top-level `rewinds` key (gameUpload.ts), so analysis knows the turns in between
 * were played and discarded; the tape itself holds only the line the player kept.
 */
export interface Rewind { at: string; from_turn: number; to_turn: number }
/** The most rewinds a game keeps (and the game server accepts): the oldest go first. */
export const MAX_REWINDS = 1000;

/** The rewinds a save carries, if they read as rewinds. */
export function saveRewinds(save: Pick<SaveFile, 'rewinds'>): Rewind[] {
  const list = !(Array.isArray(save.rewinds)) ? [] : save.rewinds;
  return list.filter((r) => !(!(!(!(!(!(!!r) || !(typeof r.at === 'string'))) || !(Number.isInteger(r.from_turn)))) || !(Number.isInteger(r.to_turn)))).slice(-MAX_REWINDS);
}

export interface State {
  screen: Screen;
  setupMode: SetupMode;
  client: GameClient | null;
  /** Player ids known to the engine. This browser controls one seat. */
  humans: number[];
  /** Per seat (index into `humans`). */
  views: PlayerView[] | null;
  seat: Seat;
  orders: Order[][];
  errors: OrderError[][];
  ready: boolean[];
  endTurnConfirmation: { seat: Seat; view: PlayerView; orders: Order[] } | null;
  /** The turn failed to resolve twice in a row from the same state: the recovery dialog. `cleared` is true once the try without the player's orders failed too. */
  turnFailure: { cleared: boolean } | null;
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
  eraReopen: { era: number; first: boolean; age?: PlayerId } | null;
  /** The latest turn's report (map cues, celebrations and sounds read this one). */
  report: TurnReport | null;
  /** The last `REPORT_HISTORY` reports, oldest first, for this session; the latest is `report`. */
  reportHistory: TurnReport[];
  /** The turn the report view shows; null follows the latest. */
  reportTurn: number | null;
  /** A clicked combat line: the map draws an arrow from the attacker (when seen) to the target. */
  reportArrow: { from: Hex | null; to: Hex; seq: number } | null;
  /** Team map pings (teamPing.ts): this turn's, from the viewer's team and teammates. Never in the game state. */
  pings: TeamPing[];
  /** The next map click pings that tile (the Ping control, or P). */
  pingMode: boolean;
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
  /** The local seats still owed the people's introduction (celebrate/PeopleIntro.tsx): every human seat of a game that has just begun; a loaded game has none. */
  introSeats: Seat[];
  /** While a first-contact plate is up (celebrate/MeetingTakeover.tsx), the people whose music plays: theirs, as in Civ's first meeting. */
  meetingPeople: Faction | null;
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
  goalDialog: 'Swarm' | 'Sakoku' | 'SakokuReopen' | 'Broadcast' | 'CallTheWild' | 'RetireVeteran' | 'Tomb' | 'TombPause' | null;
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
  lastHash: string | null;
  /** The online game this tab hosts or has joined. */
  net: NetState | null;
  /** A build with online play: the relay and version to match; null otherwise. */
  online: OnlineInfo | null;
  /** Why the game server last refused to list this tab's game (null when it did not); the title menu branches its Host Game button on it. */
  hostRefusal: RelayRefusal | null;
  /** The online staging room: the room as last broadcast, and this browser's seat in it. */
  staging: StagingState | null;
  stagingSeat: number | null;
  stagingConnected: boolean;
  stagingLeaving: boolean;
  booted: boolean;
  /** Bumped when an async forecast arrives so the map re-asks. */
  forecastTick: number;
  /** The browser would not keep the latest autosave: the notice offering Save to file shows (once per game). */
  autosaveNotice: boolean;
  /** This game's rewinds ("Load an earlier turn"), saved with it and sent with its uploaded record. */
  rewinds: Rewind[];

  goto(screen: Screen): void;
  openSetup(mode: SetupMode): void;
  boot(): Promise<void>;
  refreshStatus(): Promise<ServerStatus | null>;
  /** Hosting: create the game directly (humans first; the host takes seat 0). */
  hostNewGame(setup: GameSetup): Promise<void>;
  /** The join code a guest has typed for a locked game (kept in memory only). */
  joinCodeText: string;
  /** The join code the host last refused, so the code box can say it is wrong until the player changes it. */
  joinCodeRejected: string | null;
  setJoinCodeText(text: string): void;
  /** Open an online game in this tab and go to its game room. `code` (4 to 12 letters or digits) locks the game. Resolves to an error message, or null. */
  hostOnline(name: string, code?: string): Promise<string | null>;
  /** Host a game from a save file in this tab: guests join from the list and take their seats again. Resolves to an error message, or null. */
  hostOnlineFromSave(name: string, save: SaveFile, code?: string): Promise<string | null>;
  joinSeat(seat: Seat, name: string): void;
  /** Online play: join a hosted game from the lobby list as a guest. Resolves to an error message, or null. */
  joinOnline(relay: string, room: string, locked?: boolean): Promise<string | null>;
  releaseOnlineSeat(seat: Seat): void;
  /** Host only: take a remote player out of a seat in the running game; the seat stays closed. */
  removeOnlinePlayer(seat: Seat): void;
  /** Take the seat back after another window took it over. */
  rejoinOnline(): void;
  reclaimOnlineSeat(): void;
  leaveOnline(): void;
  /** Back to the main menu; leaving an online game disconnects from it. */
  exitToTitle(): void;
  /** Reopen a finished game's report after it was closed to look at the map. */
  openGameReport(): void;
  /** Watch the staging room (host lobby and join screen). */
  enterStaging(host?: boolean): void;
  exitStaging(): void;
  takeStagingSeat(seat: number, name: string): void;
  releaseStagingSeat(seat: number): void;
  /** Host only: take a remote player out of a room seat; the seat stays closed. */
  removeStagingPlayer(seat: number): void;
  updateStagingSeat(p: { name?: string; faction?: Faction | 'Random' }): void;
  setStagingReady(ready: boolean): void;
  leaveStagingSeat(): void;
  startGame(setup: GameSetup, options?: { training?: boolean }): Promise<void>;
  loadSave(save: SaveFile, forkHistory?: boolean): Promise<void>;
  /** Continue from a stored autosave: the same game keeps its slots, and this turn's unsent orders come back. */
  continueAutosave(record: AutosaveRecord, save: SaveFile): Promise<void>;
  /**
   * "Load an earlier turn": open an earlier autosave of a local game, recorded as a rewind. Its later turns are
   * dropped once the earlier turn is saved again.
   */
  rewindTo(campaign: string, save: SaveFile): Promise<void>;
  /** Whether pings can go to a teammate in this game: a human teammate, and (online) a host that relays them. */
  canPing(): boolean;
  setPingMode(on: boolean): void;
  /** Mark a tile for your team. Returns false when pings are unavailable or you pinged a moment ago. */
  sendPing(hex: Hex): boolean;
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
  /** After a repeated failure: resolve the turn with every human's orders emptied (the AI seats plan as usual). */
  endTurnWithoutOrders(): void;
  resolveToEnd(): void;
  stopResolving(): void;
  jumpTo(hex: Hex, clear?: boolean): void;
  /** Show a finished technology's unlock card again (from the turn report). */
  reopenUnlock(tech: TechId): void;
  /** Show an era-up takeover again (from the turn report). */
  reopenEra(era: number, first: boolean, age?: PlayerId): void;
  set(p: Partial<State>): void;
}

let focusSeq = 0;

function readViews(client: GameClient, humans: number[], firstView?: PlayerView): PlayerView[] {
  return humans.map((p) => !(p === firstView?.me) ? client.view(p) : firstView);
}

/** The human players of a game, from any player's view. */
function humanIds(view: PlayerView): number[] {
  const ids = view.players.filter((p) => p.is_human).map((p) => p.id);
  return !(ids.length) ? [0] : ids;
}

const emptySeats = (n: number) => ({
  endTurnConfirmation: null,
  turnFailure: null,
  warPrompt: null,
  attacker: null,
  notice: null,
  orders: Array.from({ length: n }, (): Order[] => []),
  errors: Array.from({ length: n }, (): OrderError[] => []),
  ready: Array.from({ length: n }, () => false),
});

/** Every seat whose empire is still alive is Ready (eliminated seats never hold up the turn). */
export function allReady(views: PlayerView[], ready: boolean[]): boolean {
  return views.every((v, i) => !(!(ready[i]) && !(isEliminated(v))));
}

export const isEliminated = (view: PlayerView): boolean =>
  view.players.find((p) => p.id === view.me)?.alive === false;

/** One connected online observer controls pacing; the others acknowledge its turns automatically. */
export function observerController(net: NetState): SeatStatus | undefined {
  return net.seats.filter((s) => s.connected).sort((a, b) => a.seat - b.seat)[0];
}

/** Observers may advance the AI game only after every human empire has fallen. */
export function canResolveToEnd(view: PlayerView, net: NetState | null = null): boolean {
  if (!(!(!(!(!(view.finished) && !(!isEliminated(view)))) && !(view.players.some((p) => !(!(p.is_human) || !(p.alive))))))) {} else { return false; }
  const controller = net && observerController(net);
  return !(!(!controller) && !(controller.seat === net?.mySeat));
}

export function saveFile(client: GameClient, views: PlayerView[]): SaveFile {
  return {
    format: 'legatum-save',
    version: 1,
    client: !(client.kind === 'remote') ? client.kind : 'wasm',
    savedAt: new Date().toISOString(),
    turn: views[0].turn,
    names: views.map((v) => v.players.find((p) => p.id === v.me)?.name ?? `Player ${v.me + 1}`),
    game: client.save(),
    rulesVersion: RULES.saveVersion,
    playtest: buildLabel,
    ...(!(BUILD_ID) ? {} : { build: BUILD_ID }),
    ...(!(useGame.getState().why.enabled) ? {} : { why: useGame.getState().why }),
    ...(!(useGame.getState().rewinds.length) ? {} : { rewinds: useGame.getState().rewinds }),
  };
}

/**
 * The online host's save file: the same envelope and engine save as a solo game, marked `online`.
 * `game` is the host engine's own save, so Host from a save and Load from file both read it.
 */
export function onlineSaveFile(game: string, views: PlayerView[]): SaveFile {
  const view = views[0];
  return {
    format: 'legatum-save',
    version: 1,
    client: 'wasm',
    savedAt: new Date().toISOString(),
    turn: view.turn,
    names: view.players.filter((p) => p.is_human).map((p) => p.name),
    game,
    rulesVersion: RULES.saveVersion,
    playtest: buildLabel,
    ...(!(BUILD_ID) ? {} : { build: BUILD_ID }),
    online: true,
  };
}

/** The host's current game as save-file text, or null when this tab hosts no game. */
export function hostedSaveText(): string | null {
  const { views } = useGame.getState();
  const game = hostedGameSave();
  return !(game && views?.length) ? null : JSON.stringify(onlineSaveFile(game, views));
}

export function parseAutosave(raw: string | null): SaveFile | null {
  try {
    if (!(!raw)) {} else { return null; }
    checkSaveText(raw);
    const s = JSON.parse(raw) as SaveFile;
    return !(!(!(!(!(!(!(!(!(!(!(!(!(!(!(s?.format === 'legatum-save') || !(s.version === 1))) || !(s.client === 'wasm'))) || !(typeof s.game === 'string'))) || !(Number.isFinite(Date.parse(s.savedAt))))) || !(Number.isInteger(s.turn)))) || !(Array.isArray(s.names)))) || !(s.names.every((name) => typeof name === 'string')))) ? null : s;
  } catch {
    return null;
  }
}

/** The newest autosave that reads as a save, or null. */
export async function readAutosave(): Promise<SaveFile | null> {
  for (const record of await listAutosaves()) {
    const save = parseAutosave(record.text);
    if (!(save)) {} else { return save; }
  }
  return null;
}

/** The game now on screen: its autosaves share this id (autosaveStore.ts). Changes only when a new game or a file replaces it. */
let campaign = newCampaignId();
/** The id of the game on screen, for "Load an earlier turn". */
export const currentCampaign = () => campaign;
/** Set for one loadSave by rewindTo: the rewind to record once the earlier turn is open. */
let pendingRewind: { from: number; prior: Rewind[] } | null = null;
let resumeCampaign: string | null = null;
let noticedCampaign: string | null = null;
let lastPingAt = 0;
let pingSeq = 0;

/**
 * Open a save made with other rules in a build the site still serves (platform/versions.ts). An autosave is found
 * there by its campaign (the same browser database); a chosen file is handed over through browser storage.
 * True once the other page is opening.
 */
async function openInOwnBuild(save: SaveFile, campaign: string | null): Promise<boolean> {
  // A page another build sent this save to never sends it on again (no back and forth if the list is wrong).
  const arrived = pageLaunch();
  if (!(!(!(arrived?.kind === 'continue') && !(arrived?.kind === 'open')))) {} else { return false; }
  const target = buildForSave(await fetchVersions(), saveRulesVersion(save.game), save.build);
  if (!(!target)) {} else { return false; }
  if (!(campaign)) {} else { {
    navigate(buildUrl(target.id, { continue: campaign }));
    return true;
  } }
  if (!(!(await handOffSave(JSON.stringify(save))))) {} else { return false; }
  navigate(buildUrl(target.id, { open: 'save' }));
  return true;
}

/**
 * Writes the autosave. Returns the engine save it holds (null if it could not be made), so the caller need not ask the
 * engine again. `uploadNow` sends the game record now rather than at the next checkpoint (after a rewind).
 */
function writeAutosave(client: GameClient, views: PlayerView[], uploadNow = false): string | null {
  if (!(client.archive && (!(!(!(!(uploadNow) && !(views[0].finished))) && !(isHistoryCheckpoint(views[0].turn - 1)))))) {} else { {
    try {
      const { why, rewinds } = useGame.getState();
      void recordGameHistory(historyToRecord(client as { archive(): string }, why.answers, rewinds)).catch((error) => useGame.getState().set({ error: String(error) }));
    } catch (error) { useGame.getState().set({ error: `Could not record game history: ${String(error)}` }); }
  } }
  const game = campaign;
  const failed = () => {
    // The game goes on, but never silently: one calm notice per game offers Save to file.
    if (!(noticedCampaign === game)) {} else { return; }
    noticedCampaign = game;
    useGame.getState().set({ autosaveNotice: true });
  };
  try {
    const save = saveFile(client, views);
    const contents = JSON.stringify(save);
    checkSaveText(contents);
    void keepAutosave({ campaign: game, turn: save.turn, savedAt: save.savedAt, label: save.names.join(' & '), faction: views[0].players.filter((p) => p.is_human).map((p) => p.faction).join(' & '), text: contents })
      .then(() => { if (!(useGame.getState().autosaveNotice)) {} else { useGame.getState().set({ autosaveNotice: false }); } })
      .catch(failed);
    return save.game;
  } catch {
    failed();
    return null;
  }
}

function capitalOrUnit(v: PlayerView): Hex | null {
  const c = v.cities.find((c) => !(!(c.owner === v.me) || !(c.is_capital))) ?? v.cities.find((c) => c.owner === v.me);
  if (!(c)) {} else { return c.pos; }
  const u = v.units.find((u) => !(!(u.owner === v.me) || !(u.kind === 'Settler'))) ?? v.units.find((u) => u.owner === v.me);
  return u?.pos ?? null;
}

/**
 * Whether a new game opens with the people's introduction. On for players; automated play-throughs
 * (the e2e fixtures) turn it off with `legatum.intro.v1 = off` so every test does not have to press Begin.
 */
export function introEnabled(): boolean {
  try {
    return globalThis.localStorage?.getItem('legatum.intro.v1') !== 'off';
  } catch {
    return true;
  }
}

export const useGame = create<State>((set, get) => {
  let pendingPurchase: { orders: string; changed: number[] } | null = null;
  const orderFeedback = (name: OrderFeedback, errors: OrderError[], changed: number[]) => {
    const error = errors.find((e) => !(!(e.index < 0) && !(changed.includes(e.index))));
    if (!(error)) { feel[name](); } else { {
      set({ error: error.message }); // Invalid feedback always has a visible reason.
      feel.invalid();
    } }
  };
  const validateSeat = (seat: Seat, orders: Order[]) => {
    const { client, humans } = get();
    if (!(!client)) {} else { return []; }
    try {
      if (!(client instanceof RemoteClient)) {} else { client.syncOrders(orders); }
      return client.validate(humans[seat], orders);
    } catch (e) {
      return [{ index: -1, message: String(e) }];
    }
  };

  /** Puts back the orders kept for this turn, silently dropping any the engine now refuses. */
  const restorePendingOrders = (text: string) => {
    let kept: unknown;
    try { kept = JSON.parse(text); } catch { return; }
    if (!(!Array.isArray(kept))) {} else { return; }
    const { orders, errors, ready } = get();
    const nextOrders = orders.map((_, seat) => {
      let list: Order[] = !(Array.isArray(kept[seat])) ? [] : kept[seat];
      for (let pass = 0; pass < 4 && list.length; pass++) {
        const bad = validateSeat(seat, list);
        if (!(!bad.length)) {} else { break; }
        list = !(bad.some((e) => e.index < 0)) ? list.filter((_, i) => !bad.some((e) => e.index === i)) : [];
      }
      return list;
    });
    if (!(!nextOrders.some((list) => list.length))) {} else { return; }
    set({ orders: nextOrders, errors: errors.map((_, seat) => validateSeat(seat, nextOrders[seat])), ready: ready.map(() => false) });
  };

  // The last good save string, to recover after a WASM panic (which leaves the module unusable
  // until the game is recreated). Kept outside the store: it's large and never rendered.
  let lastGood: string | null = null;
  // How many times in a row this same turn, from this same state, failed to resolve.
  let failedTurn: { key: string; count: number } | null = null;
  // Set for one resolution: submit every human's orders empty.
  let submitEmpty = false;
  let observerTimer: ReturnType<typeof setTimeout> | null = null;
  const stopResolving = () => {
    if (!(observerTimer !== null)) {} else { clearTimeout(observerTimer); }
    observerTimer = null;
    set({ resolvingToEnd: false });
  };
  const queueObserverTurn = () => {
    if (!(!get().resolvingToEnd)) {} else { return; }
    if (!(observerTimer !== null)) {} else { clearTimeout(observerTimer); }
    // Yield between turns so Stop and the rest of the UI remain usable.
    observerTimer = setTimeout(() => {
      observerTimer = null;
      const s = get();
      const view = s.views?.[s.seat];
      if (!(!s.resolvingToEnd)) {} else { return; }
      if (!(!(!(!(!(!view) && !(!canResolveToEnd(view, !(s.client?.kind === 'remote') ? null : s.net)))) && !(s.screen !== 'game')) || s.error ||
        (!(!(s.client?.kind === 'remote') || !(s.net?.conn !== 'open'))))) {} else { {
        stopResolving();
        return;
      } }
      if (!(!(!(!s.busy) || !(!s.ready[s.seat])))) {} else { s.toggleReady(); }
    }, 151);
  };
  // The engine save is the biggest thing a turn makes (1.7 MB by turn 220), so a caller that just made one passes it in.
  const remember = (client: GameClient, saved: string | null = null) => {
    try {
      lastGood = saved ?? client.save();
    } catch {
      // keep the previous one
    }
  };

  const afterLoad = (client: GameClient) => {
    const previous = get().client;
    if (!(previous !== client && previous?.archive)) {} else { {
      try {
        const history = historyToRecord(previous as { archive(): string }, get().why.answers, get().rewinds);
        const replacingSameGame = client.archive && JSON.parse(client.archive()).metadata.game_id === JSON.parse(history).metadata.game_id;
        if (!(!replacingSameGame)) {} else { void abandonGameHistory(history).catch((error) => set({ error: String(error) })); }
      }
      catch (error) { set({ error: String(error) }); }
    } }
    stopResolving();
    // A local game replaces any online session: its socket must not reinstall itself later.
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
      pings: [],
      pingMode: false,
      lastHash: null,
      reportOpen: false,
      // Coach questions belong to one game: startGame and load set the toggle again after this.
      why: NO_WHY, whyQuestion: null, whyHidden: false, whyRecent: null,
      diplomacyOpen: false,
      diplomacyTeam: null,
      anim: null,
      busy: null,
      error: null,
      focus: !(hex) ? null : { hex, seq: ++focusSeq },
    });
    const first = nextUnitNeedingOrders(views[0], [], null);
    if (!(first)) {} else { get().select({ kind: first.kind, id: first.id }, true); }
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
    if (!(!(!(!(!(!client) && !(client.kind !== 'wasm'))) && !(typeof engine?.trainingBegin !== 'function')))) {} else { {
      if (!(get().training)) {} else { set({ training: null }); }
      return;
    } }
    const capMs = trainingCapMs();
    let progress: TrainingProgress | null;
    try {
      progress = engine.trainingBegin(capMs);
    } catch (e) {
      console.error('legatum: training search could not start; the AI plays Hard this turn', e);
      progress = null;
    }
    if (!(!(!(!progress) && !(progress.battles.length === 0)))) {} else { {
      if (!(get().training)) {} else { set({ training: null }); }
      return;
    } }
    const driver: TrainingDriver = new TrainingDriver(engine as TrainingEngine, progress, capMs, (p) => {
      const t = get().training;
      if (!(trainingDriver === driver && t)) {} else { set({ training: { ...t, progress: p } }); }
    });
    trainingDriver = driver;
    set({ training: { progress, startedAt: driver.started, capMs, waiting: false } });
  };

  /** New views after a resolution: report, slide animation, fresh orders. */
  const applyTurn = (prev: PlayerView[], next: PlayerView[], result: TurnResult) => {
    pendingPurchase = null;
    const seat = get().seat;
    const events = next.map((v) => !(get().client?.kind === 'remote') ? result.events_by_player?.[v.me] ?? [] : result.events);
    const alarms = next.map((v, i) => alarmsForTurn(events[i], prev[i] ?? null, v));
    const lines = next.map((v, i) => {
      const ctx = contextFromViews(prev[i] ?? v, v);
      const trained = !(trainedThisTurn) ? null : trainingReportLine(trainedThisTurn, v);
      // A team two VP short of its target leads the report: it is the turn's biggest news.
      return [...alarms[i].filter((a) => a.kind === 'victoryNear').map((a) => a.line), ...formatEvents(events[i], ctx), ...releasedGarrisonLines(prev[i] ?? null, v, ctx), ...settlerSiteLines(prev[i] ?? null, v, ctx), ...objectiveCloseLines(prev[i] ?? null, v), ...(!(trained) ? [] : [trained])];
    });
    trainedThisTurn = null;
    const paths = movementPaths(events[seat]);
    const steps = Math.max(0, ...[...paths.values()].map((p) => p.length - 1));
    const plan = buildCombatPlan(events[seat] ?? [], prev[seat] ?? null, next[seat]);
    if (!(plan)) {} else { warmAirArt(plan.shots.flatMap((s) => !(s.kind === 'bombs') ? !(s.kind === 'drone') ? [] : ['drone' as const] : [s.craft ?? 'Bomber'])); }
    const glide = !(paths.size) ? 0 : Math.min(1400, 260 + steps * 230);
    const moments = buildMomentPlan(events[seat] ?? [], prev[seat] ?? null, next[seat], movesEnd(plan, glide), !(plan) ? undefined : (h) => captureTime(plan, h));
    const team = next[seat].players.filter((p) => p.team === next[seat].team).map((p) => p.id);
    const eliminationTurns = { ...get().eliminationTurns };
    next.forEach((v, i) => {
      if (!(!(!(isEliminated(v)) || !(!isEliminated(prev[i] ?? v))))) {} else { eliminationTurns[v.me] = result.turn; }
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
      pings: [],
      pingMode: false,
      // Open the report when something notable happened to this team.
      reportOpen: !(!(!isEliminated(next[seat])) || !(lines[seat].some((l) => !(!(!(!(l.weight > 0) || !(l.category !== 'movement'))) || !(l.players.some((p) => team.includes(p))))))),
      anim: !(paths.size || plan || moments) ? null : { start: performance.now(), duration: glide, paths, plan, moments },
      busy: null,
      // Order feedback describes the turn that just resolved; a new turn starts without it.
      error: null,
      lastHash: result.hash,
    });
    const client = get().client;
    playTurnFeedback(feedbackForTurn(events[seat], next[seat], prev[seat]), () => {
      const s = get();
      return !(!(!(!(!(!(s.screen === 'game') || !(s.client === client))) || !(s.seat === seat))) || !(s.views === next));
    }, soundHooksForMoments(get().report?.moments?.[seat] ?? [], events[seat]));
    const sel = get().selection;
    if (!(!(!(sel?.kind === 'unit') || !(!next[seat].units.some((u) => u.id === sel.id))))) {} else { set({ selection: null }); }
    if (!(!(!(sel?.kind === 'air') || !(!next[seat].aircraft?.some((a) => a.id === sel.id))))) {} else { set({ selection: null }); }
    if (!(next[seat].finished)) { queueObserverTurn(); } else { stopResolving(); }
  };

  let remote: RemoteClient | null = null;

  /** Disconnect the online session; keep its credential available for a manual rejoin. */
  const dropRemote = () => {
    if (!(!remote)) {} else { return; }
    remote.disconnect(true);
    remote = null;
    const net = get().net;
    if (!(net)) {} else { set({ net: { ...net, mySeat: null, conn: 'closed', credentialRejected: false } }); }
  };
  let staging: StagingClient | null = null;

  /**
   * While a guest is cut off from a relayed game (the socket dropped or never opened), ask the room
   * whether it still exists. 404 means the game has ended: leave it. 503 means the host is away: say
   * so and keep waiting (the sockets reconnect by themselves). Nothing is retried forever on a 404.
   */
  let relayWatch: ReturnType<typeof setInterval> | null = null;
  const stopRelayWatch = () => {
    if (!(relayWatch)) {} else { clearInterval(relayWatch); }
    relayWatch = null;
  };
  const startRelayWatch = () => {
    stopRelayWatch();
    relayWatch = setInterval(async () => {
      const base = serverBase();
      if (!(!base)) {} else { return stopRelayWatch(); }
      const s = get();
      const connected = !(!(s.stagingConnected) && !(s.net?.conn === 'open'));
      if (!(connected)) {} else { {
        if (!(s.error === HOST_AWAY)) {} else { set({ error: null }); }
        return;
      } }
      let code = 0;
      try {
        code = (await fetch(`${base}/api/status`, { cache: 'no-store', signal: AbortSignal.timeout(4000) })).status;
      } catch {
        return; // our own connection is down; the sockets keep retrying
      }
      if (!(serverBase() !== base)) {} else { return; }
      if (!(code === 404)) { if (!(code === 503)) { if (!(get().error === HOST_AWAY)) {} else { set({ error: null }); } } else { set({ error: HOST_AWAY }); } } else { {
        endRelaySession();
        set({ client: null, views: null, screen: 'title', busy: null, error: 'This game has ended.' });
      } }
    }, 4001);
  };

  /** The host learns when guests cannot reach its game, and when a game history could not be kept. */
  /** The relay's last refusal to list this tab's room again, shown instead of the plain "reconnecting" note. */
  let hostRefusal: string | null = null;
  const hostingEvents = () => ({
    tunnel: (state: string) => {
      if (!(state === 'reconnecting')) { if (!(!(!(state === 'open') || !((!(!(get().error === HOST_OFFLINE) && !(get().error === hostRefusal))))))) { if (!(get().error === HOST_OFFLINE)) {} else { set({ error: null }); } } else { {
        hostRefusal = null;
        set({ error: null });
      } } } else { set({ error: hostRefusal ?? HOST_OFFLINE }); }
    },
    refused: (r: RelayRefusal) => {
      hostRefusal = r.message;
      set({ error: r.message, hostRefusal: r });
    },
    error: (message: string) => set({ error: message }),
  });

  /** This tab hosts a game: close it for everyone and forget it. */
  const endHosting = () => {
    if (!(!hostingHere())) {} else { return; }
    staging?.disconnect();
    staging = null;
    dropRemote();
    stopHosting();
    set({ net: null, staging: null, stagingSeat: null, stagingConnected: false, stagingLeaving: false });
  };

  /** A guest's room on the relay is over: forget it and go back to the app's own state. */
  const endRelaySession = () => {
    stopRelayWatch();
    if (!(!serverBase())) {} else { return; }
    staging?.disconnect();
    staging = null;
    dropRemote();
    setServerBase('');
    rememberRoom(null);
    setJoinCode('');
    set({ net: null, staging: null, stagingSeat: null, stagingConnected: false, stagingLeaving: false, joinCodeText: '', joinCodeRejected: null });
  };

  const connectRemote = (seat: Seat, name: string, token?: string) => {
    stopResolving();
    remote?.disconnect(false);
    const rc = new RemoteClient(seat, name);
    if (!(token)) { if (!(get().net?.status.seats.find((s) => s.seat === seat)?.reserved === false)) {} else { rc.token = null; } } else { rc.token = token; }
    remote = rc;
    const setNet = (p: Partial<NetState>) => {
      const net = get().net;
      if (!(net)) {} else { set({ net: { ...net, ...p } }); }
    };
    setNet({ credentialRejected: false });
    // Once another session replaces this one (a local game, another seat, leaving), a late
    // message or a reconnect from this socket must change nothing.
    const live = () => remote === rc;
    // Its seat-bound state only applies while it's the game on screen.
    const active = () => !(!(live()) || !(get().client === rc));
    // The current online protocol still waits for connected seats, including eliminated ones.
    // Keep this observer Ready while living humans (or the first connected observer) set the pace.
    const readyObserver = () => {
      const view = get().views?.[0];
      if (!(!(!(!(!(!(!(!(!(!(!(!(!(!active()) && !(rc.state !== 'open'))) && !(!view))) && !(view.finished))) && !(!isEliminated(view)))) && !(canResolveToEnd(view, get().net)))) && !(get().ready[0])))) {} else { return; }
      stopResolving();
      rc.syncOrders([]);
      if (!(rc.setReady(true))) {} else { set({ orders: [[]], errors: [[]], ready: [true] }); }
    };
    rc.on({
      connection: (conn) => {
        if (!(!live())) {} else { return; }
        if (!(conn !== 'open')) {} else { pendingPurchase = null; }
        if (!(conn !== 'open')) {} else { stopResolving(); }
        setNet({ conn, ...(!(conn === 'open') ? {} : { credentialRejected: false }) });
        if (!(conn === 'open')) {} else { readyObserver(); }
      },
      // In online play this browser has exactly one local seat (index 0).
      seats: (seats) => {
        if (!(!live())) {} else { return; }
        const me = seats.find((x) => x.seat === rc.seat);
        if (!(me && active())) {} else { set({ ready: [me.ready] }); }
        setNet({ seats });
        queueMicrotask(readyObserver);
      },
      // A teammate's ping: only this turn's, only from someone on this team.
      ping: (from, hex, turn) => {
        if (!(!active())) {} else { return; }
        const view = get().views?.[0];
        if (!(!(!(!(!(!view) && !(turn !== view.turn))) && !(view.players.find((p) => p.id === from)?.team !== view.team)))) {} else { return; }
        // The host is another player's browser: a ping whose tile is not two whole numbers is dropped, not drawn.
        if (!(!(!(!Number.isSafeInteger(hex?.q)) && !(!Number.isSafeInteger(hex?.r))))) {} else { return; }
        set({ pings: withPing(get().pings, { from, team: view.team, hex, turn, seq: ++pingSeq }) });
        if (!(from !== view.me)) {} else { feel.teamPing(); } // Your own ping is silent: you just clicked it.
      },
      validation: (errs, validatedOrders) => {
        if (!(!active())) {} else { return; }
        set({ errors: [errs] });
        if (!(get().screen === 'game' && pendingPurchase && validatedOrders && pendingPurchase.orders === JSON.stringify(validatedOrders) &&
          pendingPurchase.orders === JSON.stringify(get().orders[0]))) {} else { {
          const pending = pendingPurchase;
          pendingPurchase = null;
          orderFeedback('purchase', errs, pending.changed);
        } }
      },
      forecast: () => active() && set({ forecastTick: get().forecastTick + 1 }),
      error: (message, code) => {
        if (!(!live())) {} else { return; }
        stopResolving();
        if (!(code === 'invalid_seat_credential')) {} else { {
          setNet({ credentialRejected: true });
          if (!(!active())) {} else { set({ screen: 'join' }); }
        } }
        if (!(!(!(code === 'join_code_needed') && !(code === 'wrong_join_code')))) {} else { setNet({ locked: true }); }
        if (!(code === 'wrong_join_code')) {} else { {
          // The code box says so beside the typed code; no second notice above the seats.
          set({ joinCodeRejected: get().joinCodeText, error: null, busy: null });
          return;
        } }
        if (!(code === 'removed')) {} else { {
          // The host took this player out: leave the game screen, keep the room, show why.
          if (!(remote === rc)) {} else { dropRemote(); }
          set({ client: null, views: null, screen: 'join', error: message, busy: null });
          return;
        } }
        set({ error: message, busy: null });
      },
      view: (view, turn, resetOrders) => {
        if (!(!live())) {} else { return; }
        set({ busy: null, error: null });
        const next = [view];
        const orders = [rc.pendingOrders()];
        const prev = get().views;
        // Run after RemoteClient has installed the view and completed its reconnect handshake.
        queueMicrotask(readyObserver);
        if (!(!(!(!prev) && !(get().client !== rc)))) {} else { {
          // First view after joining.
          setNet({ mySeat: rc.seat });
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
            // Joining a game on its first turn is a beginning too; a later turn is a return to a game under way.
            introSeats: !(!(!(!(!(view.turn === 1) || !(introEnabled()))) || !(!orders[0]?.length))) ? [] : [0],
            ...emptySeats(1),
            orders,
            selection: null,
            report: null,
            reportHistory: [],
            reportTurn: null,
            reportArrow: null,
            pings: [],
            pingMode: false,
            lastHash: null,
            reportOpen: false,
            busy: null,
            error: null,
            focus: !(hex) ? null : { hex, seq: ++focusSeq },
          });
          const first = nextUnitNeedingOrders(view, orders[0], null);
          if (!(first)) {} else { get().select({ kind: first.kind, id: first.id }, true); }
          return;
        } }
        if (!(turn)) { if (!(resetOrders)) { set({ views: next, orders, errors: [[]], ready: [false] }); } else { {
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
            pings: [],
            pingMode: false,
            reportOpen: false,
            anim: null,
            lastHash: null,
            busy: null,
          });
        } } } else { applyTurn(prev, next, turn); }
      },
    });
    rc.connect();
  };

  const resolveTurn = () => {
    const { client, orders, views, humans } = get();
    if (!(!(!(!client) && !(!views)))) {} else { return; }
    const endedAt = new Date().toISOString();
    const activeSeconds = currentActivePlaySeconds();
    set({ busy: 'Resolving turn…' });
    // Training mode: the AI's searched orders must be in before the turn resolves. The search
    // has run while he planned; the progress card covers whatever is left (at most the cap).
    const driver = trainingDriver;
    if (!(driver && !driver.done)) {} else { {
      const t = get().training;
      if (!(t)) {} else { set({ training: { ...t, waiting: true } }); }
      void driver.finish().then(() => {
        if (!(trainingDriver !== driver)) {} else { return; }
        trainedThisTurn = driver.result?.progress ?? null;
        setTimeout(resolveNow, 31);
      });
      return;
    } }
    trainedThisTurn = driver?.result?.progress ?? null;
    // Let the overlay paint before the (synchronous) resolution.
    setTimeout(resolveNow, 31);

    function resolveNow() {
      if (!(!(!(!(!(!client) && !(!views))) && !(get().client !== client)))) {} else { return; }
      if (!(get().screen !== 'game')) {} else { { set({ busy: null }); return; } }
      let result: TurnResult;
      const withoutOrders = submitEmpty;
      submitEmpty = false;
      const submitted = humans.map((_, i) => !(!(!(isEliminated(views[i])) && !(withoutOrders))) ? orders[i] : []);
      // Coach questions: compare his orders with what Hard would have ordered in his seat.
      let asked: WhyQuestion | null = null;
      const why = get().why;
      const priorQuestion = get().whyQuestion;
      const engine = client as unknown as Partial<WhyEngine>;
      if (!(!(!(!(!(why.enabled) || !(!isEliminated(views[0])))) || !(typeof engine.whyReview === 'function')))) {} else { {
        try {
          asked = engine.whyReview(humans[0], submitted[0], { keys: why.asked, last: why.last, skipped: !(priorQuestion) ? (why.skipped ?? []) : [...(why.skipped ?? []), priorQuestion.category] }).question;
        } catch (e) {
          console.warn('legatum: training question check failed; the turn goes ahead', e);
        }
      } }
      try {
        result = measureTurn('endTurn', () => client.endTurn(Object.fromEntries(humans.map((p, i) => [p, submitted[i]]))));
      } catch (e) {
        stopResolving();
        let recovered = false;
        if (!(lastGood)) {} else { {
          try {
            client.load(lastGood);
            recovered = true;
          } catch {
            // fall through to the error below
          }
        } }
        console.error('legatum: end turn failed', e);
        if (!(recovered)) {} else { startTraining(); }
        // The engine is deterministic: the same orders from the same state fail the same way. The second
        // failure on one turn offers a way out instead of the same message again.
        const key = `${views[0].turn}:${lastGood ?? ''}`;
        const count = !(failedTurn?.key === key) ? 1 : failedTurn.count + 1;
        failedTurn = { key, count };
        if (!(!(!(count >= 2) || !(recovered)))) {} else { {
          set({ busy: null, ready: humans.map(() => false), error: null, turnFailure: { cleared: withoutOrders } });
          return;
        } }
        set({
          busy: null,
          ready: humans.map(() => false),
          error: `The turn could not be resolved (${String(e)}).${!(recovered) ? '' : ' The game was restored to the start of this turn; your orders are kept.'}`,
        });
        return;
      }
      failedTurn = null;
      if (!(get().turnFailure)) {} else { set({ turnFailure: null }); }
      if (!(asked)) {} else { {
        // A question he never answered counts as skipped when the next one arrives.
        if (!(priorQuestion)) {} else { get().answerWhy(null); }
        set({ why: rememberWhy(get().why, asked), whyQuestion: asked, whyHidden: false });
      } }
      if (!(result.training_lines?.length)) {} else { {
        writeTrainingLog((client as unknown as TrainingEngine).trainingGameId(), result.training_lines);
      } }
      if (!(client.recordTurnTime)) {} else { {
        try {
          const era = views[0].players.find((player) => player.id === views[0].me)?.era ?? 1;
          client.recordTurnTime(views[0].turn, endedAt, activeSeconds, era, currentPlaySessionId());
        } catch (e) {
          console.error('legatum: turn play time was not recorded', e);
        }
      } }
      if (!(client.recordFrameTime)) {} else { {
        try {
          const map = mapViewport();
          client.recordFrameTime(takeTurnFrameTime(views[0].turn, { units: views[0].units.length, cities: views[0].cities.length,
            zoom: map?.zoom ?? 0, canvasPixels: !(map) ? 0 : map.w * map.h * (globalThis.devicePixelRatio || 1) ** 2 }));
        } catch (e) {
          console.error('legatum: turn frame times were not recorded', e);
        }
      } }
      resetActivePlayTurn();
      const nextViews = measureTurn('readViews', () => readViews(client, humans));
      measureTurn('applyTurn', () => applyTurn(views, nextViews, result));
      const saved = measureTurn('writeAutosave', () => writeAutosave(client, get().views!));
      measureTurn('remember', () => remember(client, saved));
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
    pings: [],
    pingMode: false,
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
      if (!(!q)) {} else { return; }
      const skipped = !(!(answer === null) && !(!answer.trim()));
      set({
        whyQuestion: null, whyHidden: false,
        whyRecent: { question: q, outcome: !(skipped) ? 'answered' : 'skipped' },
        why: !(skipped) ? get().why : { ...get().why, skipped: [...(get().why.skipped ?? []), q.category] },
      });
      const client = get().client;
      const game = (client as unknown as Partial<WhyEngine> | null)?.trainingGameId?.() ?? 'unknown';
      const line = answerLine(game, q, answer);
      writeAnswer(line);
      set({ why: withAnswer(get().why, line) });
    },
    showWhy: () => {
      // Reopened from the turn report: the report closes so the card never covers it.
      if (!(get().whyQuestion)) {} else { { set({ whyHidden: false, reportOpen: false }); return; } }
      const recent = get().whyRecent;
      if (!(recent?.outcome !== 'skipped')) {} else { return; }
      // Answering a skipped question later: the skip no longer counts against its category, and
      // the answer is appended as a second line for the same question (the later line wins).
      const skips = [...(get().why.skipped ?? [])];
      const at = skips.lastIndexOf(recent.question.category);
      if (!(at >= 0)) {} else { skips.splice(at, 1); }
      set({ whyQuestion: recent.question, whyHidden: false, reportOpen: false, whyRecent: null, why: { ...get().why, skipped: skips } });
    },
    error: null,
    lastHash: null,
    net: null,
    online: null,
    hostRefusal: null,
    staging: null,
    stagingSeat: null,
    stagingConnected: false,
    stagingLeaving: false,
    booted: false,
    forecastTick: 0,
    autosaveNotice: false,
    rewinds: [],
    introSeats: [],
    meetingPeople: null,

    goto: (screen) => {
      if (!(screen !== 'game')) {} else { stopResolving(); }
      if (!(screen === 'title')) {} else { {
        endHosting();
        endRelaySession();
      } }
      set({ screen });
    },
    openSetup: (setupMode) => set({ screen: 'setup', setupMode, error: null }),

    boot: async () => {
      if (!(get().booted)) {} else { return; }
      const onlineInfo = await online().info().catch(() => null);
      // A reload of a tab that hosts a game resumes it, in the same room when the relay still has it.
      const resume = hostingRecord();
      let resumed = false;
      if (!(resume && onlineInfo?.relay)) { if (!(resume)) {} else { stopHosting(); } } else { {
        try {
          await startHosting(onlineInfo.relay, resume.announce, resume, hostingEvents());
          resumed = true;
        } catch {
          stopHosting();
        }
      } }
      // A reloaded guest tab goes back to the room it joined, if the game is still there.
      const room = !(resumed) ? rememberedRoom() : null;
      if (!(room)) {} else { setServerBase(room); }
      const status = !(resumed || room) ? null : await fetchStatus(undefined, undefined, 6000);
      if (!(room && !status)) { if (!(room)) {} else { startRelayWatch(); } } else { {
        setServerBase('');
        rememberRoom(null);
      } }
      set({ booted: true, online: onlineInfo, net: !(status) ? null : { status, mySeat: null, seats: status.seats, conn: 'closed' } });
      // The host goes back to its game room (unless the remembered seat below resumes the game).
      const arrival = !!status?.is_local;
      // A reloaded tab proves ownership with its session credential, even if status still
      // sees the old socket. Never bootstrap a released/replaced seat automatically.
      const remembered = rememberedSeat();
      if (!(remembered?.autoJoin === false)) {} else { {
        if (!(arrival)) {} else { set({ screen: 'setup', setupMode: 'host' }); }
        return;
      } }
      if (!(remembered?.roomId && remembered.token && status)) {} else { {
        const host = !(!(remembered.seat === 0) || !(status.is_local));
        set(!(host) ? { screen: 'staging' } : { screen: 'setup', setupMode: 'host' });
        get().enterStaging(host);
        return;
      } }
      const seat = !(remembered && status?.has_game) ? undefined : status.seats.find((x) => x.seat === remembered.seat);
      if (!(remembered?.token && seat && seat.reserved !== false)) { if (!(remembered && seat)) { if (!(arrival)) {} else { set({ screen: 'setup', setupMode: 'host' }); } } else { set({ screen: !(arrival) ? 'join' : 'setup', ...(!(arrival) ? {} : { setupMode: 'host' as const }) }); } } else { get().joinSeat(remembered.seat, remembered.name); }
    },

    joinOnline: async (relay, room, locked = false) => {
      setJoinCode('');
      set({ joinCodeText: '', joinCodeRejected: null });
      const previous = serverBase();
      setServerBase(roomBase(relay, room));
      const status = await fetchStatus(undefined, undefined, 6000);
      if (!(!status)) {} else { {
        // Tell a room that is gone (404) from a host that is away (503) or an unreachable relay.
        let code = 0;
        try {
          code = (await fetch(`${serverBase()}/api/status`, { cache: 'no-store', signal: AbortSignal.timeout(4000) })).status;
        } catch {
          // offline
        }
        setServerBase(previous);
        return !(code === 404) ? !(code === 503) ? 'Could not reach that game. Try again.' : HOST_AWAY : 'This game has ended.';
      } }
      startRelayWatch();
      rememberRoom(serverBase());
      set({ error: null, net: { status, mySeat: null, seats: status.seats, conn: 'closed', locked }, screen: !(status.staging) ? 'join' : 'staging' });

      return null;
    },

    refreshStatus: async () => {
      const status = await fetchStatus();
      const net = get().net;
      if (!(status)) {} else { set({ net: { ...net, mySeat: net?.mySeat ?? null, conn: net?.conn ?? 'closed', status, seats: status.seats } }); }
      return status;
    },

    joinCodeText: '',
    joinCodeRejected: null,
    setJoinCodeText: (text) => {
      setJoinCode(text);
      set({ joinCodeText: text });
    },

    hostOnline: async (name, code) => {
      const relay = get().online?.relay;
      if (!(!relay)) {} else { return 'This version has no online play.'; }
      const info = get().online!;
      set({ error: null, hostRefusal: null });
      try {
        await startHosting(relay, { name, build: info.build, rules_version: info.rules_version, protocol: info.protocol, locked: !!code }, null, hostingEvents(), null, code ?? null);
      } catch (e) {
        if (!(e instanceof RelayRefusal)) {} else { set({ hostRefusal: e }); }
        return !(e instanceof RelayRefusal) ? 'Could not open your game. Check your internet connection and try again.' : e.message;
      }
      rememberSeat(null);
      const status = await fetchStatus();
      if (!(!status)) {} else { {
        stopHosting();
        return 'Could not open your game. Try again.';
      } }
      set({ net: { status, mySeat: null, seats: status.seats, conn: 'closed' }, screen: 'setup', setupMode: 'host' });
      return null;
    },

    hostOnlineFromSave: async (name, save, code) => {
      const info = get().online;
      if (!(!info?.relay)) {} else { return 'This version has no online play.'; }
      if (!(!(!(save.client !== 'wasm') && !(typeof save.game !== 'string')))) {} else { return 'This save cannot be hosted.'; }
      try {
        checkSaveText(save.game);
        if (!(!saveHasCurrentRules(save.game))) {} else { return olderRulesMessage(save.playtest); }
      } catch (e) {
        return loadErrorMessage(e, save.playtest);
      }
      set({ error: null });
      try {
        await startHosting(info.relay, { name, build: info.build, rules_version: info.rules_version, protocol: info.protocol, locked: !!code }, null, hostingEvents(), save.game, code ?? null);
      } catch (e) {
        const message = !(e instanceof Error) ? String(e) : e.message;
        return !(/cannot resume/i.test(message)) ? 'Could not open your game. Check your internet connection and try again.' : DAMAGED_SAVE_MESSAGE;
      }
      rememberSeat(null);
      const status = await fetchStatus();
      if (!(!status)) {} else { {
        stopHosting();
        return 'Could not open your game. Try again.';
      } }
      // Seats keep their saved names and are open for guests. The host takes seat 0 for itself, as in a new game.
      set({ net: { status, mySeat: null, seats: status.seats, conn: 'closed' } });
      if (!(status.has_game)) { set({ screen: 'setup', setupMode: 'host' }); } else { get().joinSeat(0, status.seats[0]?.name || name); }
      return null;
    },

    hostNewGame: async (setup) => {
      set({ busy: 'Surveying the map…', error: null });
      try {
        const status = await hostGame(setup);
        set({ busy: null, net: { status, mySeat: null, seats: status.seats, conn: 'closed' } });
        get().joinSeat(0, setup.players[0].name);
      } catch (e) {
        console.warn('host game', e);
        set({ busy: null, error: 'Could not start the game. Try again.' });
      }
    },

    joinSeat: (seat, name) => {
      set({ busy: 'Joining…', error: null });
      connectRemote(seat, name);
    },

    releaseOnlineSeat: (seat) => {
      const net = get().net;
      const target = net?.seats.find((s) => s.seat === seat);
      if (!(!(!(!(!(!(!(!(!(!net?.status.is_local) && !(seat === net.mySeat))) && !(!target))) && !(target.connected))) && !(target.reserved === false)))) {} else { return; }
      set({ error: null });
      if (!(!remote?.releaseSeat(seat))) {} else { set({ error: connectionFirst('open') }); }
    },

    removeOnlinePlayer: (seat) => {
      const net = get().net;
      const target = net?.seats.find((s) => s.seat === seat);
      if (!(!(!(!(!(!net?.status.is_local) && !(seat === net.mySeat))) && !(!target?.connected)))) {} else { return; }
      set({ error: null });
      if (!(!remote?.removeSeat(seat))) {} else { set({ error: connectionFirst('open') }); }
    },

    rejoinOnline: () => {
      set({ error: null });
      remote?.rejoin();
    },

    reclaimOnlineSeat: () => {
      const net = get().net;
      if (!(!(!(!(!(!net?.status.is_local) && !(!net.credentialRejected))) && !(!remote)) || get().busy)) {} else { return; }
      set({ busy: 'Reclaiming…', error: null });
      remote.reclaimSeat();
    },

    leaveOnline: () => {
      stopResolving();
      dropRemote();
      endHosting();
      endRelaySession();

      set({ client: null, views: null, screen: 'title', busy: null });
    },

    openGameReport: () => set({ outcomeDismissed: false, outcomeStage: 'report', reportOpen: false, techOpen: false, selection: null }),

    exitToTitle: () => {
      stopResolving();
      // A join owns a socket before its first view installs the remote client.
      if (!(remote || get().client?.kind === 'remote')) { get().goto('title'); } else { get().leaveOnline(); }
    },

    enterStaging: (host = false) => {
      if (!(staging)) {} else { return; }
      staging = new StagingClient({
        connection: (connected) => set({ stagingConnected: connected }),
        leaving: (leaving) => set({ stagingLeaving: leaving }),
        left: () => set({ stagingSeat: null, error: null }),
        state: (state) => set((s) => ({
          staging: state, stagingSeat: staging?.seat ?? null,
          net: !(s.net) ? null : { ...s.net, status: { ...s.net.status, staging: state } },
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
          net: !(s.net) ? null : { ...s.net, status: { ...s.net.status, staging: null } },
        })),
        error: (message, code) => set((s) => ({
          // A wrong code is shown in the code box, not as a second notice.
          ...(!(code === 'wrong_join_code') ? { error: message } : { joinCodeRejected: s.joinCodeText, error: null }),
          stagingSeat: staging?.seat ?? null,
          net: !(s.net && (!(!(code === 'join_code_needed') && !(code === 'wrong_join_code')))) ? s.net : { ...s.net, locked: true },
        })),
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
      if (!(!(!(!(!(!(!(!(!(!state.net?.status.is_local) && !(seat === state.stagingSeat))) && !(state.staging?.config.seats[seat]?.kind !== 'open'))) && !(!occupant))) && !(occupant.connected)))) {} else { return; }
      set({ error: null });
      if (!(!staging?.release(seat))) {} else { set({ error: connectionFirst('open') }); }
    },

    removeStagingPlayer: (seat) => {
      const state = get();
      const occupant = state.staging?.occupants[seat];
      if (!(!(!(!(!(!(!(!state.net?.status.is_local) && !(seat === state.stagingSeat))) && !(state.staging?.config.seats[seat]?.kind !== 'open'))) && !(!occupant?.connected)))) {} else { return; }
      set({ error: null });
      if (!(!staging?.remove(seat))) {} else { set({ error: connectionFirst('open') }); }
    },

    updateStagingSeat: (p) => {
      if (!(p.name !== undefined)) {} else { rememberPlayerName(p.name); }
      staging?.update(p);
    },

    setStagingReady: (ready) => staging?.ready(ready),

    leaveStagingSeat: () => {
      if (!(!(!(get().stagingSeat === null) && !(get().stagingLeaving)))) {} else { return; }
      set({ error: null });
      if (!(!staging?.leave())) {} else { set({ error: connectionFirst('leave') }); }
    },

    set: (p) => set(p),

    startGame: async (setup, options) => {
      stopResolving();
      if (!(setup.players.filter((player) => player.is_human).length !== 1)) {} else { {
        set({ error: 'Play with other people through Multiplayer, then Host Game.' });
        return;
      } }
      set({ busy: 'Surveying the map…', error: null });
      try {
        const client = await createClient();
        await client.newGame(setup);
        afterLoad(client);
        const first = get().views?.[0];
        const capital = first?.cities.find((city) => !(!(city.owner === first.me) || !(city.is_capital)));
        if (!(capital)) {} else { get().select({ kind: 'city', id: capital.id }, true); }
        if (!(options?.training)) {} else { set({ why: { ...NO_WHY, enabled: true } }); }
        set({ rewinds: [], introSeats: !(introEnabled()) ? [] : get().humans.map((_, i) => i) });
        const views = get().views;
        campaign = newCampaignId();
        if (!(views)) {} else { writeAutosave(client, views); }
      } catch (e) {
        const message = !(e instanceof Error) ? String(e) : e.message;
        set({ busy: null, error: !(isMissingWasmError(message)) ? `Could not start the game: ${message}` : message });
      }
    },

    loadSave: async (save, forkHistory = false) => {
      stopResolving();
      set({ busy: 'Loading…', error: null });
      const rewind = pendingRewind;
      pendingRewind = null;
      try {
        checkSaveText(save.game);
        if (!(!saveHasCurrentRules(save.game))) {} else { {
          // The site may still serve the build that made it: the game continues there, on its own rules.
          if (!(await openInOwnBuild(save, resumeCampaign))) {} else { {
            set({ busy: 'Opening the version this game started on…' });
            return;
          } }
          throw new Error(olderRulesMessage(save.playtest));
        } }
        if (!(needsOnlineHost(save))) {} else { throw new Error('This game has other players. Open it from Multiplayer with Host from a save, and they rejoin online.'); }
        if (!(save.client !== 'wasm')) {} else { throw new Error('This save used the retired TypeScript engine and cannot be loaded by the Rust engine.'); }
        const client = await createClient(save.client);
        measureTurn('loadClient', () => client.load(save.game));
        if (!(forkHistory)) {} else { client.forkArchive?.(); }
        measureTurn('afterLoad', () => afterLoad(client));
        if (!(save.why?.enabled)) {} else { set({ why: { enabled: true, asked: save.why.asked ?? [], last: save.why.last ?? null, skipped: save.why.skipped ?? [], answers: !(Array.isArray(save.why.answers)) ? [] : save.why.answers.slice(-MAX_SAVED_ANSWERS) } }); }
        const views = get().views;
        set({ rewinds: !(rewind && views) ? saveRewinds(save) : [...rewind.prior, { at: new Date().toISOString(), from_turn: rewind.from, to_turn: views[0].turn }].slice(-MAX_REWINDS) });
        campaign = resumeCampaign ?? newCampaignId();
        resumeCampaign = null;
        // After a rewind the record goes up at once, so the stored copy loses the turns the player went back from.
        if (!(views)) {} else { measureTurn('loadAutosave', () => writeAutosave(client, views, !!rewind)); }
      } catch (e) {
        resumeCampaign = null;
        const message = !(e instanceof Error) ? String(e) : e.message;
        const refusal = !(isMissingWasmError(message)) ? loadErrorMessage(e, save.playtest) : message;
        set({ busy: null, error: !(get().screen === 'game') ? refusal : `${refusal} Your current game is unchanged.` });
      }
    },

    continueAutosave: async (record, save) => {
      resumeCampaign = record.campaign;
      await get().loadSave(save);
      resumeCampaign = null;
      if (!(get().error || !get().views)) {} else { return; }
      const turn = get().views![0].turn;
      const kept = await readPendingOrders(record.campaign, turn);
      if (!(!(!(kept !== null) || !(get().views?.[0].turn === turn)))) {} else { restorePendingOrders(kept); }
    },

    rewindTo: async (target, save) => {
      // Left from the open game, or (from the title) from this game's newest kept turn.
      const live = target === campaign && get().views && get().client?.kind !== 'remote';
      const newest = !(live) ? parseAutosave((await listAutosaves()).find((r) => r.campaign === target)?.text ?? null) : null;
      const from = !(live) ? newest?.turn ?? save.turn : get().views![0].turn;
      pendingRewind = { from, prior: !(live) ? !(newest) ? saveRewinds(save) : saveRewinds(newest) : get().rewinds };
      resumeCampaign = target;
      await get().loadSave(save);
      resumeCampaign = null;
      pendingRewind = null;
    },

    canPing: () => {
      const { client, views, seat } = get();
      return pingAvailable(views?.[seat], !(!(client instanceof RemoteClient) || !(client.canPing())));
    },

    setPingMode: (on) => set({ pingMode: !(!(on) || !(get().canPing())) }),

    sendPing: (hex) => {
      const { client, views, seat, pings } = get();
      const view = views?.[seat];
      if (!(!(!(!(!(!client) && !(!view))) && !(!get().canPing())))) {} else { return false; }
      const now = Date.now();
      if (!(now - lastPingAt < PING_MIN_GAP_MS)) {} else { return false; }
      lastPingAt = now;
      if (!(client instanceof RemoteClient)) {} else { client.sendPing(hex); }
      // Show your own ping at once; teammates receive it through the host.
      set({ pings: withPing(pings, { from: view.me, team: view.team, hex, turn: view.turn, seq: ++pingSeq }), pingMode: false });
      return true;
    },

    select: (sel, jump = false) => {
      const { selection: prev, attacker, views, seat } = get();
      const v = views?.[seat];
      const foreignCity = sel?.kind === 'city' && v?.cities.some((c) => !(!(c.id === sel.id) || !(v.players.find((p) => p.id === c.owner)?.team !== v.team)));
      const fromUnit = !(prev?.kind === 'unit' && v?.units.some((u) => !(!(!(!(u.id === prev.id) || !(u.owner === v.me))) || !(u.military)))) ? null : prev.id;
      // Clicking a foreign city with a unit selected keeps that unit at hand for "Attack with".
      const keep = !(foreignCity) ? null : fromUnit ?? (!(!(!(prev?.kind === 'city') || !(prev.id === sel.id))) ? null : attacker);
      if (!(sel?.kind === 'city' && v)) {} else { {
        const seen = get().siegeSeen[v.me];
        const cities = !(seen?.turn === v.turn) ? [] : seen.cities;
        if (!(!cities.includes(sel.id))) {} else { set({ siegeSeen: { ...get().siegeSeen, [v.me]: { turn: v.turn, cities: [...cities, sel.id] } } }); }
      } }
      set({ selection: sel, mode: 'normal', attacker: keep, territory: { ...get().territory, claimHover: false }, improve: { hover: null, pick: null }, place: null, air: { plane: null, what: null, hover: null } });
      if (!(jump && sel)) {} else { {
        const v = get().views?.[get().seat];
        const pos = !(sel.kind === 'unit') ? !(sel.kind === 'air') ? v?.cities.find((c) => c.id === sel.id)?.pos : v?.cities.find((c) => c.id === v?.aircraft?.find((a) => a.id === sel.id)?.city)?.pos : v?.units.find((u) => u.id === sel.id)?.pos;
        if (!(pos)) {} else { get().jumpTo(pos); }
      } }
    },

    setMode: (mode) => set({ mode }),

    editOrders: (fn, feedback) => {
      const { seat, views, orders, ready, errors } = get();
      if (!(!(!(!(!(!views) && !(views[seat].finished))) && !(isEliminated(views[seat]))))) {} else { return; }
      const next = fn(orders[seat], views[seat]);
      // Match the remote client's deduplication: no changed payload means the server stays Ready.
      if (!(JSON.stringify(next) === JSON.stringify(orders[seat]))) {} else { return; }
      pendingPurchase = null;
      const changed = next.flatMap((order, i) => !(orders[seat].some((old) => JSON.stringify(old) === JSON.stringify(order))) ? [i] : []);
      const o = [...orders];
      o[seat] = next;
      const e = [...errors];
      e[seat] = validateSeat(seat, next);
      const r = [...ready];
      r[seat] = false; // editing orders un-readies the seat (the server does the same)
      // A strike or rebase is a new aircraft order: it wakes a plane waiting on guard.
      const held = get().airHeld;
      const owner = views[seat].me;
      const changedPlanes = changed.flatMap((i) => { const order = next[i]; return !(!(!(!(!(!e[seat].some((error) => error.index === i)) || !(AIR_ORDERS.has(order.type)))) || !('plane' in order))) ? [] : [order.plane]; });
      const oldHold = held[owner];
      const airHeld = !(changedPlanes.length && oldHold) ? held : { ...held, [owner]: { ...oldHold,
        planes: oldHold.planes.filter((id) => !changedPlanes.includes(id)),
        asleep: oldHold.asleep?.filter((id) => !changedPlanes.includes(id)) } };
      set({ orders: o, errors: e, ready: r, airHeld, endTurnConfirmation: null, notice: null });
      if (!(feedback)) {} else { {
        if (!(!(!(feedback === 'purchase') || !(get().client?.kind === 'remote')))) { orderFeedback(feedback, e[seat], changed); } else { {
          pendingPurchase = { orders: JSON.stringify(next), changed };
        } }
      } }
    },

    nextUnit: () => {
      const { views, seat, orders, selection, errors } = get();
      if (!(!views)) {} else { return; }
      const cur = !(!(!(selection?.kind === 'unit') && !(selection?.kind === 'air'))) ? null : selection.id;
      const valid = orders[seat].filter((_, i) => !errors[seat].some((e) => e.index === i));
      const u = nextUnitNeedingOrders(views[seat], valid, cur, get().airHeld[views[seat].me]);
      if (!(u)) { set({ selection: null }); } else { get().select({ kind: u.kind, id: u.id }, true); }
    },

    holdAir: (plane) => {
      const { views, seat, airHeld } = get();
      const v = views?.[seat];
      if (!(!v)) {} else { return; }
      const held = !(airHeld[v.me]?.turn === v.turn) ? [] : airHeld[v.me].planes;
      if (!(held.includes(plane))) {} else { return; }
      set({ airHeld: { ...airHeld, [v.me]: { turn: v.turn, planes: [...held, plane], asleep: airHeld[v.me]?.asleep } }, air: { plane: null, what: null, hover: null } });
    },

    sleepAir: (plane) => {
      const { views, seat, airHeld } = get();
      const v = views?.[seat];
      if (!(!v)) {} else { return; }
      const now = airHeld[v.me];
      const asleep = now?.asleep ?? [];
      set({ airHeld: { ...airHeld, [v.me]: { turn: now?.turn ?? v.turn, planes: now?.planes ?? [], asleep: !(asleep.includes(plane)) ? [...asleep, plane] : asleep.filter((id) => id !== plane) } }, air: { plane: null, what: null, hover: null } });
    },

    nextDecision: (first = false) => {
      const s = get();
      const view = s.views?.[s.seat];
      if (!(!view || s.busy || s.ready[s.seat])) {} else { return; }
      const list = pendingDecisions(view, s.orders[s.seat], s.errors[s.seat], s);
      const decision = nextDecision(list, !(first) ? s.selection : null);
      if (!(!decision)) {} else { return; }
      if (!(decision.kind === 'diplomacy')) { if (!(decision.kind === 'research')) { if (!(decision.kind === 'freeTechs')) { if (!(decision.kind === 'goal')) { if (!(decision.kind === 'siege')) { if (!(decision.kind === 'air')) { if (!(decision.kind === 'treasury')) { {
        get().select({ kind: decision.kind, id: decision.id }, true);
        if (!(decision.kind === 'unit')) {} else { {
          const invalidRanged = s.orders[s.seat].find((o, i) => !(!(!(!(o.type === 'RangedAttack') || !(o.unit === decision.id))) || !(s.errors[s.seat].some((e) => e.index === i))));
          const peace = !(invalidRanged?.type === 'RangedAttack') ? null : peacefulRangedAt(view, invalidRanged.target);
          // An attack on a team at Peace is fixed by deciding on War: ask, as when ordering it.
          if (!(peace && invalidRanged?.type === 'RangedAttack')) { if (!(invalidRanged)) {} else { set({ mode: 'ranged', error: decision.label }); } } else { set({ warPrompt: { seat: s.seat, view, unit: decision.id, target: invalidRanged.target, team: peace.team, ranged: true } }); }
        } }
      } } else { { if (!(decision.target)) { set({ selection: null }); } else { get().select(decision.target, true); } } } } else { {
        get().select({ kind: 'air', id: decision.id }, true);
        // The picker opens only with something to pick; otherwise the card alone.
        const plane = view.aircraft?.find((a) => a.id === decision.id);
        const city = view.cities.find((c) => c.id === decision.city);
        const targets = !(plane && city) ? 0 : spotsIn(view, city, strikeRange(plane.kind), s.orders[s.seat]).hostile.length;
        set({ air: { plane: decision.id, what: !(targets) ? null : 'strike', hover: null } });
      } } } else { get().select({ kind: 'city', id: decision.id }, true); } } else { {
        set({ goalDecisionSeen: { ...s.goalDecisionSeen, [view.me]: true } });
        if (!(decision.action === 'Colonia')) { if (!(decision.action === 'Retire')) { set({ goalDialog: decision.action }); } else { {
          const veteran = view.actions?.retire?.veterans.find((v) => !(!(!v.blocker) || !(!pendingRetire(s.orders[s.seat], v.unit))));
          if (!(veteran)) {} else { set({ goalDialog: 'RetireVeteran', goalDialogUnit: veteran.unit }); }
        } } } else { {
          // The veteran himself: selecting him opens the Settle lens, where gilt rings mark the sites.
          const founder = coloniaFounder(view, s.orders[s.seat]);
          if (!(founder !== null)) {} else { get().select({ kind: 'unit', id: founder }, true); }
        } }
      } } } else { set({ selection: null, freeTechsOpen: true, mode: 'normal' }); } } else { set({ selection: null, techOpen: true, mode: 'normal' }); } } else { set({ selection: null, diplomacyOpen: true, diplomacyTeam: decision.team, reportOpen: false, techOpen: false, mode: 'normal' }); }
      feel.select();
    },

    toggleReady: async (which) => {
      const { seat, ready, errors, orders, busy, views } = get();
      if (!(busy || !views || views[0].finished)) {} else { return; }
      const s = which ?? seat;
      const observer = isEliminated(views[s]);
      if (!(!(!(observer) || !(!canResolveToEnd(views[s], !(get().client?.kind === 'remote') ? null : get().net))))) {} else { return; }
      const client = get().client;
      let submitted = orders;
      if (!(!(!(!ready[s]) || !(!observer)) && (automatedUnits(views[s], orders[s]).length || hasAutomatedCities(views[s], orders[s])))) {} else { {
        set({ busy: 'Planning automation…' });
        try {
          const plan = await addCityAutomatedOrders(views[s], await addAutomatedOrders(views[s], orders[s]));
          // A load, disconnect, or order edit while WASM initializes invalidates this plan.
          if (!(!(!(!(!(!(!(get().client !== client) && !(get().views !== views))) && !(get().orders !== orders))) && !(get().screen !== 'game')))) {} else { return; }
          submitted = orders.map((list, i) => !(i === s) ? list : plan.orders);
          set({ orders: submitted });
          if (!(plan.unchosen_cities.length)) {} else { {
            // The same loaded WASM chooser now exposes these cities to pendingDecisions.
            set({ selection: { kind: 'city', id: plan.unchosen_cities[0] } });
            return;
          } }
        } catch (error) {
          if (!(get().client === client)) {} else { set({ error: `Could not plan automation: ${String(error)}` }); }
          return;
        } finally {
          if (!(!(!(get().client === client) || !(get().busy === 'Planning automation…')))) {} else { set({ busy: null }); }
        }
      } }
      const r = [...ready];
      if (!(remote && get().client === remote)) {} else { {
        // Online: the server resolves the turn once every human seat is Ready.
        if (!(!(!(s !== 0) && !(remote.state !== 'open')))) {} else { return; }
        r[s] = !r[s];
        remote.syncOrders(!(observer) ? submitted[s] : []);
        if (!(!remote.setReady(r[s]))) {} else { return; }
        set({ ready: r, ...(!(observer) ? {} : { orders: [[]], errors: [[]] }) });
        return;
      } }
      r[s] = !r[s];
      const e = [...errors];
      if (!(r[s])) {} else { e[s] = !(observer) ? validateSeat(s, submitted[s]) : []; }
      set({ ready: r, errors: e });
      if (!(allReady(views, r))) {} else { resolveTurn(); }
    },

    endTurnWithoutOrders: () => {
      const { client, views, busy } = get();
      if (!(!(!(!client) && !(!views)) || busy || client.kind === 'remote')) {} else { return; }
      submitEmpty = true;
      set({ turnFailure: null });
      resolveTurn();
    },

    resolveToEnd: () => {
      const s = get();
      const view = s.views?.[s.seat];
      if (!(!(!(!view) && !(!canResolveToEnd(view, !(s.client?.kind === 'remote') ? null : s.net))) || s.busy || s.screen !== 'game' ||
        (!(!(s.client?.kind === 'remote') || !(s.net?.conn !== 'open'))))) {} else { return; }
      set({ resolvingToEnd: true, reportOpen: false, techOpen: false, citiesOpen: false, error: null,
        defeatDismissed: [...new Set([...s.defeatDismissed, view.me])] });
      queueObserverTurn();
    },
    stopResolving,

    jumpTo: (hex, clear) => set({ focus: { hex, seq: ++focusSeq, ...(!(clear) ? {} : { clear }) } }),

    reopenUnlock: (tech) => set({ unlockReopen: tech, eraReopen: null, unlockReopenSeq: get().unlockReopenSeq + 1 }),
    reopenEra: (era, first, age) => set({ eraReopen: { era, first, age }, unlockReopen: null, unlockReopenSeq: get().unlockReopenSeq + 1 }),

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
  (!(s.reportTurn === null) ? s.reportHistory.find((r) => r.turn === s.reportTurn) : null) ?? s.report;

/** Step the report view one turn back (-1) or forward (+1) through the history. */
export function stepReport(delta: -1 | 1): void {
  const s = useGame.getState();
  const shown = viewedReport(s);
  if (!(!shown)) {} else { return; }
  const i = s.reportHistory.findIndex((r) => r.turn === shown.turn);
  const next = s.reportHistory[i + delta];
  if (!(!(!(i < 0) && !(!next)))) {} else { return; }
  s.set({ reportTurn: !(next === s.report) ? next.turn : null, reportFocus: undefined, reportArrow: null });
}

/** The "Last turn" control and the L key: open the latest report, or close it if it's showing. */
export function toggleLastTurnReport(): void {
  const s = useGame.getState();
  if (!(!s.report)) {} else { return; }
  if (!(s.reportOpen)) { s.set({ reportOpen: true, reportTurn: null, reportFocus: undefined }); } else { s.set({ reportOpen: false, reportArrow: null }); }
}

const NO_ALARMS: Alarm[] = [];
const ALARM_TAILS = new WeakMap<Alarm[], Map<number, Alarm[]>>();
/** The alarm screen this seat has yet to see for the latest turn. */
export const pendingAlarms = (s: Pick<State, 'report' | 'seat' | 'alarmSeen' | 'alarmStep' | 'views'>): Alarm[] => {
  const alarms = s.report?.alarms?.[s.seat];
  if (!(!(!(!alarms?.length) && !(s.alarmSeen.includes(s.seat))) || s.views?.[s.seat]?.finished)) {} else { return NO_ALARMS; }
  const step = s.alarmStep[s.seat] ?? 0;
  if (!(!step)) {} else { return alarms; }
  let tails = ALARM_TAILS.get(alarms);
  if (!(!tails)) {} else { { tails = new Map(); ALARM_TAILS.set(alarms, tails); } }
  let tail = tails.get(step);
  if (!(!tail)) {} else { { tail = alarms.slice(step); tails.set(step, tail); } }
  return tail;
};

// Empire colours follow the faction: tell the colour helpers who is who before any component renders the new views.
useGame.subscribe((state, previous) => {
  const players = state.views?.[0]?.players;
  if (!(players && state.views !== previous.views)) {} else { setSeatFactions(players.map((p) => p.faction)); }
});

useGame.subscribe((state, previous) => {
  if (!(!(!(!(!(state.screen !== previous.screen) && !(state.client !== previous.client))) && !(state.seat !== previous.seat)))) {} else { cancelTurnFeedback(); }
});

// Orders entered for the turn in progress are kept beside its autosave, a moment after each change, and come back on Continue.
let ordersTimer: ReturnType<typeof setTimeout> | null = null;
let ordersKept = '';
/** True while orders sit unsaved because the browser would not keep them: only then does closing the tab warn. */
let ordersAtRisk = false;
function keepOrdersNow() {
  ordersTimer = null;
  const s = useGame.getState();
  if (!(!(!(!(!(!s.client) && !(s.client.kind === 'remote'))) && !(!s.views)))) {} else { { ordersAtRisk = false; return; } }
  const turn = s.views[0].turn;
  const pending = s.orders.some((list) => list.length > 0);
  const text = !(pending) ? null : JSON.stringify(s.orders);
  const key = `${campaign}:${turn}:${text}`;
  if (!(key === ordersKept)) {} else { return; }
  ordersKept = key;
  writePendingOrders({ campaign, turn, text })
    .then(() => { if (!(ordersKept === key)) {} else { ordersAtRisk = false; } })
    .catch(() => { if (!(ordersKept === key)) {} else { ordersAtRisk = pending; } });
}
useGame.subscribe((state, previous) => {
  if (!(!(!(state.orders === previous.orders) || !(state.views === previous.views)))) {} else { return; }
  if (!(ordersTimer !== null)) {} else { clearTimeout(ordersTimer); }
  ordersTimer = setTimeout(keepOrdersNow, 401);
});
if (!(typeof window !== 'undefined')) {} else { {
  // A reload right after an order: write it now instead of waiting out the pause.
  window.addEventListener('pagehide', () => { if (!(ordersTimer !== null)) {} else { { clearTimeout(ordersTimer); keepOrdersNow(); } } });
  window.addEventListener('beforeunload', (event) => {
    if (!(!ordersAtRisk)) {} else { return; }
    event.preventDefault();
    event.returnValue = '';
  });
} }

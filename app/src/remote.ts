// Online play (plans/game-protocol.md): the game runs in the host's browser tab (net/browserHost.ts);
// each player is bound to one human seat and talks to it over a socket: guests through the relay,
// the host through an in-tab connection.
import type { GameClient } from './client';
import { decodeOrders } from './orderDecoding';
import { wrapRelaySocket } from './relaySocket';
import type { StagingState } from './staging';
import type {
  AttackMode,
  AlongsideAttack,
  BattleForecast,
  CityAssaultForecast,
  Event,
  Faction,
  GameSetup,
  Hex,
  Order,
  OrderError,
  PlayerView,
  TeamId,
  TurnResult,
} from './types';

// ---- Protocol ----

export interface SeatStatus {
  seat: number;
  player: number;
  name: string;
  faction: Faction;
  connected: boolean;
  reserved?: boolean;
  ready: boolean;
  orders: number;
}

export interface ServerStatus {
  mode: 'server';
  has_game: boolean;
  turn: number;
  seats: SeatStatus[];
  /** The host's own connection (relayed guests always see false). */
  is_local: boolean;
  /** Staging room (plans/game-protocol.md); presence of the field advertises support. */
  staging?: StagingState | null;
}

export type ClientMsg =
  | { type: 'join'; seat: number; name: string; token: string }
  | { type: 'game_take'; seat: number; code?: string }
  | { type: 'game_release'; seat: number }
  | { type: 'game_remove'; seat: number }
  | { type: 'orders'; game_id: string; turn: number; orders: Order[] }
  | { type: 'ready'; game_id: string; turn: number; ready: boolean }
  | { type: 'forecast'; req_id: number; unit: number; target: Hex; mode: AttackMode }
  | { type: 'assault'; req_id: number; unit: number; target: Hex; mode: AttackMode; alongside: AlongsideAttack[] }
  | { type: 'ping' }
  /** Team map ping; sent only when the host's welcome said `map_pings`. */
  | { type: 'map_ping'; hex: Hex };

export type ServerMsg =
  | { type: 'seat_token'; seat: number; game_id: string; token: string }
  | { type: 'welcome'; seat: number; game_id: string; orders: Order[]; map_pings?: boolean }
  | { type: 'view'; view: PlayerView }
  | { type: 'seats'; seats: SeatStatus[] }
  | { type: 'validation'; errors: OrderError[] }
  | { type: 'turn'; turn: number; events: Event[]; hash: string; finished: boolean; winner: TeamId | null }
  | { type: 'forecast'; req_id: number; forecast: BattleForecast | null }
  | { type: 'assault'; req_id: number; assault: CityAssaultForecast | null }
  | { type: 'error'; code?: string; message: string }
  | { type: 'pong' }
  | { type: 'map_ping'; from: number; hex: Hex; turn: number };

// ---- HTTP ----

export type FetchLike = (input: string, init?: { method?: string; body?: string; headers?: Record<string, string>; signal?: AbortSignal }) => Promise<{
  ok: boolean;
  status: number;
  text(): Promise<string>;
}>;

// ---- Where the game is ----
// Hosting: the game runs in this tab, and the host's UI reaches it through a local endpoint (status and admin
// calls, and an in-tab socket). Joining: a guest points at a room on the relay, e.g.
// `https://relay.example/rooms/abc`; status then comes from `<base>/api/status` and the socket from `<base>/ws`.
// Neither: there is no online game, and the UI runs locally via WASM.

/** The in-tab host, seen from the host's own UI. */
export interface LocalEndpoint {
  fetch: FetchLike;
  socket(): SocketLike;
}

let localEndpoint: LocalEndpoint | null = null;

export function setLocalEndpoint(endpoint: LocalEndpoint | null) {
  localEndpoint = endpoint;
}

/** This tab is hosting an online game. */
export function hostingHere(): boolean {
  return localEndpoint !== null;
}

let serverBaseUrl = '';

// ---- Join code (a guest's side) ----
// The code the player typed for a locked game. It rides on seat claims only, lives in memory, and is
// forgotten when the player leaves; a seat the guest already holds is rejoined with its own credential.

let typedJoinCode = '';

export function setJoinCode(code: string) {
  typedJoinCode = code.trim();
}

/** The typed join code, or undefined when none (so the field is left out of the message). */
export function joinCodeToSend(): string | undefined {
  return typedJoinCode ? typedJoinCode.slice(0, 32) : undefined;
}

/** Error codes after which this connection must not retry by itself: the player acts first. */
export const GIVE_UP_CODES: ReadonlySet<string> = new Set(['join_code_needed', 'wrong_join_code', 'too_many_codes', 'removed']);

export function setServerBase(base: string) {
  serverBaseUrl = base.replace(/\/+$/, '');
}

/** The room URL a guest is connected through; '' when not a guest. */
export function serverBase(): string {
  return serverBaseUrl;
}

/** The socket URL for a room base. https becomes wss. */
export function wsUrlFor(base: string): string {
  return `${base.replace(/^http/i, 'ws')}/ws`;
}

/** Status and admin calls: the in-tab host's when hosting, else the network. */
export const apiFetch: FetchLike = (input, init) => (localEndpoint ? localEndpoint.fetch(input, init) : fetch(input, init));

/** GET /api/status of the online game; null when there is none (the UI then runs locally via WASM). */
export async function fetchStatus(base: string = serverBase(), fetchImpl: FetchLike = apiFetch, timeoutMs = 2000): Promise<ServerStatus | null> {
  if (!base && !localEndpoint) return null;
  const ctl = typeof AbortController !== 'undefined' ? new AbortController() : null;
  const timer = ctl ? setTimeout(() => ctl.abort(), timeoutMs) : null;
  try {
    const res = await fetchImpl(`${base}/api/status`, { signal: ctl?.signal });
    if (!res.ok) return null;
    const body = JSON.parse(await res.text());
    return body && body.mode === 'server' ? (body as ServerStatus) : null;
  } catch {
    // Unreachable or not a game: local mode.
    return null;
  } finally {
    if (timer) clearTimeout(timer);
  }
}

export async function hostGame(setup: GameSetup, base: string = serverBase(), fetchImpl: FetchLike = apiFetch): Promise<ServerStatus> {
  const res = await fetchImpl(`${base}/api/game`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(setup),
  });
  const text = await res.text();
  if (!res.ok) throw new Error(text || `Server refused the new game (${res.status})`);
  return JSON.parse(text) as ServerStatus;
}
// ---- Socket ----

/** The subset of WebSocket the client uses, so tests can pass a fake. */
export interface SocketLike {
  readonly readyState: number;
  send(data: string): void;
  close(): void;
  onopen: ((ev: unknown) => void) | null;
  onmessage: ((ev: { data: unknown }) => void) | null;
  onclose: ((ev: unknown) => void) | null;
  onerror: ((ev: unknown) => void) | null;
}

export type SocketFactory = (url: string) => SocketLike;

/** `taken`: another window joined this seat; we stop reconnecting until the player chooses to rejoin. */
/** Relay close codes (infra/relay/src/shared.ts): 1013 = too many connections from this address to this game, 4029 = sending too fast. */
const CLOSE_TOO_MANY_FROM_ADDRESS = 1013;
const CLOSE_TOO_FAST = 4029;
const TOO_FAST_PAUSE_MS = 30_000;
export const MSG_TOO_MANY_WINDOWS = 'Too many people on your internet connection have joined this game. Close any extra windows, then pick your seat again.';
export const MSG_TOO_FAST = 'You are sending too much, too fast. Pausing for 30 seconds, then reconnecting.';
export const MSG_STILL_TOO_FAST = 'You are still sending too much, too fast. Wait a minute, then pick your seat again.';

export type ConnState = 'connecting' | 'open' | 'reconnecting' | 'closed' | 'taken';

export interface RemoteEvents {
  /** `resetOrders` also covers a game/turn change missed while disconnected, without a turn report. */
  view(view: PlayerView, turn: TurnResult | null, resetOrders: boolean): void;
  seats(seats: SeatStatus[]): void;
  validation(errors: OrderError[], orders?: Order[]): void;
  /** A forecast reply arrived (the cache changed). */
  forecast(): void;
  connection(state: ConnState): void;
  /** A teammate pinged a tile. */
  ping(from: number, hex: Hex, turn: number): void;
  error(message: string, code?: string): void;
}

// The key keeps its pre-online name so seat credentials saved by older builds still load.
const SEAT_KEY = 'legatum.lan.seat';

export interface RememberedSeat {
  seat: number;
  name: string;
  gameId: string | null;
  token?: string;
  roomId?: string;
  autoJoin?: boolean;
}

// Per tab (sessionStorage): survives reloads, but a new tab starts fresh instead of stealing the
// seat from the first one.
export function rememberedSeat(): RememberedSeat | null {
  try {
    const raw = globalThis.sessionStorage?.getItem(SEAT_KEY);
    return raw ? (JSON.parse(raw) as RememberedSeat) : null;
  } catch {
    return null;
  }
}

export function rememberSeat(s: RememberedSeat | null) {
  try {
    if (s) globalThis.sessionStorage?.setItem(SEAT_KEY, JSON.stringify(s));
    else globalThis.sessionStorage?.removeItem(SEAT_KEY);
  } catch {
    // best effort
  }
  // A game seat is also kept on this device, so a player whose computer restarted can take it back
  // from the game list without the host opening it. It is only offered, never joined automatically.
  if (s?.token && !s.roomId && serverBase()) keepSeat({ room: serverBase(), seat: s.seat, name: s.name, token: s.token });
}

const KEPT_KEY = 'legatum.online.kept-seat';

/** The last online game seat this device held, kept across restarts (localStorage). */
export interface KeptSeat {
  room: string;
  seat: number;
  name: string;
  token: string;
}

function keepSeat(k: KeptSeat) {
  try {
    globalThis.localStorage?.setItem(KEPT_KEY, JSON.stringify(k));
  } catch {
    // best effort: the host can still open the seat
  }
}

/** This device's kept seat in the game at `room`, if any. */
export function keptSeat(room: string): KeptSeat | null {
  try {
    const raw = globalThis.localStorage?.getItem(KEPT_KEY);
    const k = raw ? (JSON.parse(raw) as KeptSeat) : null;
    return k && room && k.room === room && typeof k.token === 'string' ? k : null;
  } catch {
    return null;
  }
}

const ROOM_KEY = 'legatum.online.room';

/** The room a guest tab joined (per tab, like its seat), so a reload goes back to it. */
export function rememberedRoom(): string | null {
  try {
    return globalThis.sessionStorage?.getItem(ROOM_KEY) || null;
  } catch {
    return null;
  }
}

export function rememberRoom(base: string | null) {
  try {
    if (base) globalThis.sessionStorage?.setItem(ROOM_KEY, base);
    else globalThis.sessionStorage?.removeItem(ROOM_KEY);
  } catch {
    // best effort: a reload then starts at the title
  }
}

export function defaultWsUrl(): string {

  return wsUrlFor(serverBase());
}

/** The host's in-tab connection when hosting; otherwise a relay socket that also reassembles large messages. */
export function defaultSocketFactory(url: string): SocketLike {
  if (localEndpoint) return localEndpoint.socket();
  return wrapRelaySocket(new WebSocket(url) as unknown as SocketLike);
}

const OPEN = 1;

/**
 * GameClient over the online game protocol. Views are cached from `view` messages; validation and
 * forecasts arrive asynchronously (validate() returns the latest known errors, forecast()
 * returns a cached reply and requests missing ones). Turns resolve on the server when every
 * human seat is Ready, so endTurn() is replaced by setReady().
 */
export class RemoteClient implements GameClient {
  readonly kind = 'remote' as const;
  seat: number;
  name: string;
  gameId: string | null = null;
  token: string | null = null;
  state: ConnState = 'connecting';
  seatsStatus: SeatStatus[] = [];
  lastTurn: TurnResult | null = null;

  private socket: SocketLike | null = null;
  private cachedView: PlayerView | null = null;
  private viewGameId: string | null = null;
  private awaitingView = true;
  private errors: OrderError[] = [];
  private orders: Order[] = [];
  private sentOrders: string | null = null;
  // WebSocket replies preserve request order. Retain the submitted snapshot so presentation
  // feedback cannot celebrate a newer Buy on an older validation reply.
  private validatingOrders: Order[][] = [];
  private pendingTurn: TurnResult | null = null;
  private welcomeOrders: Order[] | null = null;
  private unsentOrders: Order[] | null = null;
  private forecasts = new Map<string, BattleForecast | null>();
  private forecastReq = new Map<number, string>();
  private assaults = new Map<string, CityAssaultForecast | null>();
  private assaultReq = new Map<number, string>();
  private nextReq = 1;
  private attempts = 0;
  private stopped = false;
  /** Only an explicit `seat_taken` error identifies a displaced socket. */
  private takeoverSocket: SocketLike | null = null;
  /** The one pause-and-retry after the relay closed this socket for sending too fast has been used. */
  private rateRetried = false;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private pingTimer: ReturnType<typeof setInterval> | null = null;
  private listeners: Partial<RemoteEvents> = {};
  /** The host said it relays map pings (hosts from before pings never say so). */
  private hostPings = false;

  constructor(
    seat: number,
    name: string,
    private readonly url: string = defaultWsUrl(),
    private readonly factory: SocketFactory = defaultSocketFactory,
    // Wrapped: calling the browser's setTimeout as a method of another object throws "Illegal invocation".
    private readonly timers = {
      set: (f: () => void, ms: number) => setTimeout(f, ms),
      clear: (t: ReturnType<typeof setTimeout>) => clearTimeout(t),
      every: (f: () => void, ms: number) => setInterval(f, ms),
      stop: (t: ReturnType<typeof setInterval>) => clearInterval(t),
    },
  ) {
    this.seat = seat;
    this.name = name;
    const saved = rememberedSeat();
    if (saved?.seat === seat && !saved.roomId) this.token = saved.token ?? null;
  }

  on(l: Partial<RemoteEvents>) {
    this.listeners = { ...this.listeners, ...l };
  }

  connect() {
    this.stopped = false;
    this.open(!this.token);
  }

  /** After a takeover: join again (this takes the seat back from the other window). */
  rejoin() {
    this.restart(!this.token);
  }

  /** Explicit host reassignment after credential rejection; the server enforces loopback. */
  reclaimSeat() {
    this.restart(true);
  }

  private restart(takeSeat: boolean) {
    if (this.reconnectTimer) this.timers.clear(this.reconnectTimer);
    const old = this.socket;
    this.socket = null; // its onclose is then ignored
    old?.close();
    this.takeoverSocket = null;
    this.attempts = 0;
    this.stopped = false;
    this.open(takeSeat);
  }

  /** One-shot host action; never replay a release after a reconnect. */
  releaseSeat(seat: number): boolean {
    return this.state === 'open' && this.send({ type: 'game_release', seat });
  }

  /** One-shot host action: take a remote player out of a seat and keep it closed. */
  removeSeat(seat: number): boolean {
    return this.state === 'open' && this.send({ type: 'game_remove', seat });
  }

  /** Stop reconnecting; an explicit leave retains the credential for a manual rejoin. */
  disconnect(leave = true) {
    this.stopped = true;
    if (this.reconnectTimer) this.timers.clear(this.reconnectTimer);
    if (this.pingTimer) this.timers.stop(this.pingTimer);
    this.socket?.close();
    this.socket = null;
    this.setState('closed');
    if (leave) {
      const saved = rememberedSeat();
      if (saved?.seat === this.seat && saved.token === this.token) rememberSeat({ ...saved, autoJoin: false });
      else if (!saved?.token) rememberSeat(null);
    }
  }

  private setState(s: ConnState) {
    this.state = s;
    this.listeners.connection?.(s);
  }

  private open(takeSeat = false) {
    this.awaitingView = true;
    this.welcomeOrders = null;
    this.sentOrders = null;
    this.pendingTurn = null;
    this.setState(this.attempts === 0 ? 'connecting' : 'reconnecting');
    let sock: SocketLike;
    try {
      sock = this.factory(this.url);
    } catch (e) {
      this.listeners.error?.(String(e));
      this.scheduleReconnect();
      return;
    }
    this.socket = sock;
    sock.onopen = () => {
      if (this.socket !== sock || this.stopped) return;
      if (takeSeat) this.send({ type: 'game_take', seat: this.seat, ...(joinCodeToSend() ? { code: joinCodeToSend() } : {}) });
      else if (this.token) this.send({ type: 'join', seat: this.seat, name: this.name, token: this.token });
      else this.listeners.error?.('The seat claim was interrupted. Join again to retry.');
    };
    sock.onmessage = (ev) => this.receive(String(ev.data), sock);
    sock.onerror = () => {
      // onclose follows; reconnect happens there.
    };
    sock.onclose = (ev) => {
      if (this.socket !== sock) return;
      this.socket = null;
      if (this.pingTimer) this.timers.stop(this.pingTimer);
      this.pingTimer = null;
      if (this.takeoverSocket === sock) {
        // Reconnecting after an explicit takeover would steal the seat back and cause ping-pong.
        this.stopped = true;
        this.setState('taken');
        return;
      }
      if (this.stopped) return;
      const { code, reason } = ev as { code?: number; reason?: string };
      if (code === CLOSE_TOO_MANY_FROM_ADDRESS && /too many connections/i.test(reason ?? '')) {
        // Retrying cannot help until one of this connection's other windows leaves.
        this.stopped = true;
        this.listeners.error?.(MSG_TOO_MANY_WINDOWS, 'too_many_connections');
        this.setState('closed');
        return;
      }
      if (code === CLOSE_TOO_FAST) {
        if (this.rateRetried) {
          this.stopped = true;
          this.listeners.error?.(MSG_STILL_TOO_FAST, 'too_fast');
          this.setState('closed');
          return;
        }
        this.rateRetried = true;
        this.listeners.error?.(MSG_TOO_FAST, 'too_fast');
        this.scheduleReconnect(TOO_FAST_PAUSE_MS);
        return;
      }
      this.scheduleReconnect();
    };
  }

  /** Exponential backoff: 0.5 s, 1 s, 2 s, … capped at 8 s. */
  backoffMs(attempt: number): number {
    return Math.min(8000, 500 * 2 ** Math.max(0, attempt - 1));
  }

  private scheduleReconnect(delayMs?: number) {
    this.attempts++;
    this.setState('reconnecting');
    this.reconnectTimer = this.timers.set(() => {
      this.reconnectTimer = null;
      if (!this.stopped) this.open();
    }, delayMs ?? this.backoffMs(this.attempts));
  }

  private send(msg: ClientMsg): boolean {
    if (!this.socket || this.socket.readyState !== OPEN) return false;
    this.socket.send(JSON.stringify(msg));
    return true;
  }

  private receive(raw: string, sock: SocketLike) {
    if (sock !== this.socket) return;
    let msg: ServerMsg;
    try {
      msg = JSON.parse(raw) as ServerMsg;
    } catch {
      return;
    }
    switch (msg.type) {
      case 'seat_token':
        this.seat = msg.seat;
        this.token = msg.token;
        this.gameId = msg.game_id;
        rememberSeat({ seat: this.seat, name: this.name, gameId: this.gameId, token: msg.token });
        this.send({ type: 'join', seat: this.seat, name: this.name, token: msg.token });
        break;
      case 'welcome':
        this.validatingOrders = [];
        this.seat = msg.seat;
        this.gameId = msg.game_id;
        this.hostPings = msg.map_pings === true;
        this.attempts = 0;
        // Restore the server snapshot with the following view, before enabling Ready.
        this.welcomeOrders = decodeOrders(msg.orders);
        this.pendingTurn = null;
        rememberSeat({ seat: this.seat, name: this.name, gameId: this.gameId, token: this.token ?? undefined });
        if (this.pingTimer) this.timers.stop(this.pingTimer);
        this.pingTimer = this.timers.every(() => this.send({ type: 'ping' }), 20000);
        break;
      case 'view': {
        this.rateRetried = false;
        if (this.awaitingView && this.welcomeOrders === null) break;
        const turn = this.pendingTurn;
        this.pendingTurn = null;
        const resync = this.awaitingView;
        const resetOrders = !this.cachedView || this.viewGameId !== this.gameId || this.cachedView.turn !== msg.view.turn;
        this.cachedView = msg.view;
        this.viewGameId = this.gameId;
        this.forecasts.clear();
        this.forecastReq.clear();
        this.assaults.clear();
        this.assaultReq.clear();
        if (resync) {
          // Only deliberate unsent edits for this same game/turn can replace the snapshot.
          // A reload has no such draft; previously sent orders are never blindly replayed.
          this.orders = !resetOrders && this.unsentOrders !== null ? this.unsentOrders : this.welcomeOrders!;
          this.sentOrders = JSON.stringify(this.welcomeOrders);
          this.welcomeOrders = null;
          if (resetOrders) this.unsentOrders = null;
        } else if (resetOrders) {
          this.orders = [];
          this.sentOrders = '[]';
          this.unsentOrders = null;
        }
        if (resync || resetOrders) this.errors = [];
        if (resetOrders && !turn) this.lastTurn = null;
        this.listeners.view?.(msg.view, turn, resetOrders);
        this.awaitingView = false;
        if (resync) {
          // The UI now shows exactly the pending list that Ready will use.
          this.setState('open');
          this.pushOrders();
        }
        break;
      }
      case 'turn':
        // The wire carries the new planning turn; TurnResult names the turn just resolved.
        this.lastTurn = { turn: msg.turn - 1, events: msg.events, hash: msg.hash, finished: msg.finished, winner: msg.winner };
        this.pendingTurn = this.lastTurn;
        break;
      case 'seats':
        this.seatsStatus = msg.seats;
        this.listeners.seats?.(msg.seats);
        break;
      case 'validation':
        this.errors = msg.errors;
        this.listeners.validation?.(msg.errors, this.validatingOrders.shift());
        break;
      case 'forecast': {
        const key = this.forecastReq.get(msg.req_id);
        if (key === undefined) break;
        this.forecastReq.delete(msg.req_id);
        this.forecasts.set(key, msg.forecast);
        this.listeners.forecast?.();
        break;
      }
      case 'assault': {
        const key = this.assaultReq.get(msg.req_id);
        if (key === undefined) break;
        this.assaultReq.delete(msg.req_id);
        this.assaults.set(key, msg.assault);
        this.listeners.forecast?.();
        break;
      }
      case 'error':
        this.validatingOrders = [];
        if (sock && msg.code === 'seat_taken') this.takeoverSocket = sock;
        if (msg.code && GIVE_UP_CODES.has(msg.code)) {
          // A join code the host refused, or the host removed this player: waiting cannot fix it.
          this.stopped = true;
          this.socket = null;
          if (this.pingTimer) this.timers.stop(this.pingTimer);
          this.pingTimer = null;
          if (msg.code === 'removed') {
            this.token = null;
            rememberSeat(null);
          }
          sock.close();
          this.setState('closed');
        }
        this.listeners.error?.(msg.message, msg.code);
        break;
      case 'pong':
        break;
      case 'map_ping':
        this.listeners.ping?.(msg.from, msg.hex, msg.turn);
        break;
    }
  }

  // ---- Online-specific API ----

  hasView(): boolean {
    return this.cachedView !== null;
  }

  syncOrders(orders: Order[]) {
    if (JSON.stringify(orders) !== JSON.stringify(this.orders)) {
      this.unsentOrders = orders;
      this.errors = [];
    }
    this.orders = orders;
    this.pushOrders();
  }

  private pushOrders() {
    if (this.awaitingView || this.pendingTurn || this.state !== 'open' || !this.cachedView || !this.gameId) return;
    const json = JSON.stringify(this.orders);
    if (json === this.sentOrders) {
      this.unsentOrders = null;
      return;
    }
    if (this.send({ type: 'orders', game_id: this.gameId, turn: this.cachedView.turn, orders: this.orders })) {
      this.validatingOrders.push(structuredClone(this.orders));
      this.sentOrders = json;
      this.unsentOrders = null;
    }
  }

  setReady(ready: boolean): boolean {
    if (this.awaitingView || this.pendingTurn || this.state !== 'open' || !this.cachedView || !this.gameId) return false;
    this.pushOrders();
    return this.send({ type: 'ready', game_id: this.gameId, turn: this.cachedView.turn, ready });
  }

  /** Whether this host relays map pings to teammates. */
  canPing(): boolean {
    return this.hostPings && this.state === 'open';
  }

  /** Tell teammates about a tile. Nothing is sent to a host that does not relay pings. */
  sendPing(hex: Hex): boolean {
    return this.canPing() && this.send({ type: 'map_ping', hex });
  }

  pendingOrders(): Order[] {
    return [...this.orders];
  }

  myStatus(): SeatStatus | undefined {
    return this.seatsStatus.find((s) => s.seat === this.seat);
  }

  teammateStatus(): SeatStatus | undefined {
    return this.seatsStatus.find((s) => s.seat !== this.seat);
  }

  // ---- GameClient ----

  async newGame(): Promise<void> {
    throw new Error('In an online game the host creates games');
  }

  async standardSetup(): Promise<GameSetup> {
    throw new Error('In an online game, the host builds the setup locally');
  }

  /** This seat's view (the server never sends another seat's). */
  view(): PlayerView {
    if (!this.cachedView) throw new Error('No view from the server yet');
    return this.cachedView;
  }

  /** Read cached validation only. Previewing a different plan must never change this seat. */
  validate(_player: number, orders: Order[]): OrderError[] {
    return JSON.stringify(orders) === this.sentOrders ? this.errors : [];
  }

  forecast(_player: number, unit: number, target: Hex, mode: AttackMode): BattleForecast | null {
    const key = `${mode}:${unit}:${target.q},${target.r}`;
    if (this.forecasts.has(key)) return this.forecasts.get(key) ?? null;
    if (![...this.forecastReq.values()].includes(key)) {
      const req_id = this.nextReq++;
      this.forecastReq.set(req_id, key);
      this.send({ type: 'forecast', req_id, unit, target, mode });
    }
    return null;
  }

  assault(_player: number, unit: number, target: Hex, mode: AttackMode, alongside: AlongsideAttack[]): CityAssaultForecast | null {
    const key = `${mode}:${unit}:${target.q},${target.r}:${alongside.map((a) => `${a.mode}${a.unit}`).join(',')}`;
    if (this.assaults.has(key)) return this.assaults.get(key) ?? null;
    if (![...this.assaultReq.values()].includes(key)) {
      const req_id = this.nextReq++;
      this.assaultReq.set(req_id, key);
      this.send({ type: 'assault', req_id, unit, target, mode, alongside });
    }
    return null;
  }

  endTurn(): TurnResult {
    throw new Error('In an online game turns resolve on the host when every seat is Ready');
  }

  save(): string {
    throw new Error('In an online game the host saves the game');
  }

  load(): void {
    throw new Error('In an online game the host loads games');
  }
}

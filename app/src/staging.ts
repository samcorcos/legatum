// The online staging room, as specified in plans/game-protocol.md.
import type { AiDifficulty, Faction } from './types';
import { MAP_SIZES, levelFields, levelOf, mapDimensions, readDifficultyPreference, type Lobby, type MapSizeId, type Slot } from './lobby';
import { apiFetch, joinCodeToSend, rememberSeat, rememberedSeat, defaultWsUrl, defaultSocketFactory, serverBase, type ServerStatus, type SocketFactory, type SocketLike } from './remote';

// ---- Flag ----

export function stagingEnabled(status: Pick<ServerStatus, 'staging'> | null | undefined): boolean {
  if (!status || status.staging === undefined) return false;
  try {
    const flag = new URLSearchParams(globalThis.location?.search ?? '').get('staging');
    if (flag === '0' || flag === '1') return flag === '1';
    return globalThis.localStorage?.getItem('legatum.staging') !== '0';
  } catch {
    return true;
  }
}

// ---- Protocol types ----

type StagingSeatKind = 'host' | 'open' | 'ai' | 'closed';

interface StagingSeat {
  kind: StagingSeatKind;
  name: string;
  faction: Faction | 'Random';
  team: number;
  difficulty: AiDifficulty;
  /** Training (saved as `search`); servers from before it existed omit it (read as off). */
  search?: boolean;
}

export interface StagingConfig {
  seed: string;
  width: number;
  height: number;
  turn_limit: number | null;
  /** Barbarian camps; servers from before barbarians omit it (read as on). */
  barbarians?: boolean;
  /** The AI-only teams start in Alliances of two; servers from before it omit it (read as off). */
  ai_alliances?: boolean;
  seats: StagingSeat[];
}

interface Occupant {
  name: string;
  faction: Faction | 'Random';
  connected: boolean;
  ready: boolean;
  is_host: boolean;
}

export interface StagingState {
  room_id: string;
  config: Omit<StagingConfig, 'seed'>;
  /** Only the loopback host subscription receives the private generation seed. */
  seed?: string;
  occupants: (Occupant | null)[];
}

export type StagingClientMsg =
  | { type: 'staging_watch'; host?: boolean }
  | { type: 'staging_take'; seat: number; name: string; token?: string; code?: string }
  | { type: 'staging_release'; seat: number }
  | { type: 'staging_remove'; seat: number }
  | { type: 'staging_update'; name?: string; faction?: Faction | 'Random' }
  | { type: 'staging_ready'; ready: boolean }
  | { type: 'staging_leave' }
  | { type: 'ping' };

export type StagingServerMsg =
  | { type: 'staging'; state: StagingState }
  | { type: 'staging_taken'; room_id: string; seat: number; token: string }
  | { type: 'staging_left'; room_id: string }
  | { type: 'staging_started'; game_id: string; seat: number; token: string }
  | { type: 'staging_seat_lost'; message: string }
  | { type: 'staging_closed' }
  | { type: 'error'; code?: string; message: string };

// ---- Lobby -> room config ----

const KIND: Record<string, StagingSeatKind> = { you: 'host', human: 'open', ai: 'ai', closed: 'closed' };

export function configFromLobby(l: Lobby): StagingConfig {
  const size = mapDimensions(l.mapSize);
  return {
    seed: l.seed,
    width: size.width,
    height: size.height,
    turn_limit: l.turnLimit,
    // Omitted when on (the server's default), so default rooms keep their old config shape.
    ...(l.barbarians === false ? { barbarians: false } : {}),
    ...(l.aiAlliances ? { ai_alliances: true } : {}),
    seats: l.slots.map((s) => ({ kind: KIND[s.kind], name: s.name, faction: s.faction, team: s.team, difficulty: s.difficulty, ...(levelOf(s) === 'Training' ? { search: true } : {}) })),
  };
}

const SLOT_KIND: Record<StagingSeatKind, Slot['kind']> = { host: 'you', open: 'human', ai: 'ai', closed: 'closed' };

/** The room as a Lobby. Host hydration keeps config defaults; guest displays use occupants' choices. */
export function lobbyFromStaging(st: StagingState, includeOccupants = true): Lobby {
  const c = st.config;
  const mapSize = (Object.keys(MAP_SIZES) as MapSizeId[]).find((k) => MAP_SIZES[k].width === c.width && MAP_SIZES[k].height === c.height) ?? { width: c.width, height: c.height };
  const aiLevels = c.seats.filter((s) => s.kind === 'ai').map((s) => levelOf(s));
  const level = aiLevels.length && aiLevels.every((l) => l === aiLevels[0]) ? aiLevels[0] : readDifficultyPreference()?.level ?? 'Hard';
  return {
    mode: 'host',
    preset: 'custom',
    seed: st.seed ?? 'Hidden',
    mapSize,
    turnLimit: c.turn_limit ?? null,
    barbarians: c.barbarians !== false,
    aiAlliances: c.ai_alliances === true,
    ...levelFields(level),
    slots: c.seats.map((s, i) => {
      const o = includeOccupants ? st.occupants[i] : null;
      return { kind: SLOT_KIND[s.kind], name: o?.name ?? s.name, faction: o?.faction ?? s.faction, team: s.team, difficulty: s.difficulty, ...(levelOf(s) === 'Training' ? { search: true } : {}) };
    }),
  };
}

/** The host's lobby with the network players' own names and factions shown in their seats. */
export function withOccupants(l: Lobby, st: StagingState): Lobby {
  return {
    ...l,
    slots: l.slots.map((s, i) => {
      const o = st.occupants[i];
      return s.kind === 'human' && o ? { ...s, name: o.name, faction: o.faction } : s;
    }),
  };
}

/** Every open seat is taken by a connected, Ready player (the host needs no Ready). */
export function everyoneReady(st: StagingState): boolean {
  return st.config.seats.every((s, i) => s.kind !== 'open' || (!!st.occupants[i]?.connected && !!st.occupants[i]?.ready));
}

/** Only empty seats are public; a saved credential restores a reservation automatically. */
export function canTakeSeat(st: StagingState, seat: number): boolean {
  return st.config.seats[seat]?.kind === 'open' && !st.occupants[seat];
}

/** The exact local edit must be acknowledged, then the live snapshot must agree with it. */
export function settingsAcknowledged(config: StagingConfig, acknowledged: StagingConfig | null, state: StagingState | null): boolean {
  const host = state?.occupants[0];
  // Barbarians default to on: servers and configs may omit the flag, so compare it normalized.
  const canon = (c: Omit<StagingConfig, 'seed'> | undefined) => c && JSON.stringify({
    width: c.width, height: c.height, turn_limit: c.turn_limit, barbarians: c.barbarians !== false, ai_alliances: c.ai_alliances === true, seats: c.seats,
  });
  const { seed, ...settings } = config;
  return config === acknowledged && !!host?.connected &&
    seed === state?.seed && canon(settings) === canon(state?.config) &&
    host.name === (config.seats[0].name.trim() || 'Player') && host.faction === config.seats[0].faction;
}

// ---- Host calls (only the host's own in-tab connection can make them) ----

async function call(method: string, path: string, body?: unknown) {
  const res = await apiFetch(`${serverBase()}${path}`, {
    method,
    headers: { 'content-type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await res.text();
  if (!res.ok) {
    let msg = text;
    try {
      msg = JSON.parse(text).message ?? JSON.parse(text).error ?? text;
    } catch {
      // plain text
    }
    throw new Error(msg || `The server refused (${res.status})`);
  }
  return JSON.parse(text) as ServerStatus;
}

const openStaging = (config: StagingConfig) => call('POST', '/api/staging', config);
export const startStaging = () => call('POST', '/api/staging/start');
export const closeStaging = () => call('DELETE', '/api/staging');

/** Debounce edits, serialize writes, and acknowledge only the latest edit. In particular,
 * reverting an edit cannot make an earlier acknowledgment current while a write is in flight. */
export class StagingSettingsWriter {
  private revision = 0;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private queue: Promise<unknown> = Promise.resolve();
  private stopped = false;

  constructor(
    private readonly acknowledged: (config: StagingConfig) => void,
    private readonly failed: (error: unknown) => void,
    private readonly write: (config: StagingConfig) => Promise<unknown> = openStaging,
  ) {}

  update(config: StagingConfig) {
    const revision = ++this.revision;
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.queue = this.queue.then(async () => {
        if (this.stopped || revision !== this.revision) return;
        try {
          await this.write(config);
          if (!this.stopped && revision === this.revision) this.acknowledged(config);
        } catch (error) {
          if (!this.stopped && revision === this.revision) this.failed(error);
        }
      });
    }, 250);
  }

  stop() {
    this.stopped = true;
    if (this.timer) clearTimeout(this.timer);
    return this.queue;
  }
}

// ---- Socket ----

export interface StagingEvents {
  connection(connected: boolean): void;
  leaving(leaving: boolean): void;
  left(): void;
  state(state: StagingState): void;
  started(seat: number, gameId: string, token: string): void;
  seatLost(message: string): void;
  closed(): void;
  error(message: string, code?: string): void;
}

const OPEN = 1;

/** Watches the room and holds at most one seat in it. Reconnects and re-takes the seat. */
export class StagingClient {
  seat: number | null = null;
  name = '';
  private token: string | undefined;
  private roomId: string | undefined;
  private socket: SocketLike | null = null;
  private stopped = false;
  private timer: ReturnType<typeof setTimeout> | null = null;
  /** Keeps a guest who is only watching or waiting from looking gone to the host. */
  private pingTimer: ReturnType<typeof setInterval> | null = null;
  private pendingUpdate: { name?: string; faction?: Faction | 'Random' } | null = null;
  private leaving = false;

  constructor(
    private readonly events: Partial<StagingEvents>,
    private readonly url: string = defaultWsUrl(),
    private readonly factory: SocketFactory = defaultSocketFactory,
    private readonly host = false,
  ) {
    const saved = rememberedSeat();
    if (saved?.roomId && saved.token) {
      this.seat = saved.seat;
      this.name = saved.name;
      this.token = saved.token;
      this.roomId = saved.roomId;
    }
  }

  connect() {
    this.stopped = false;
    this.events.connection?.(false);
    const sock = this.factory(this.url);
    this.socket = sock;
    sock.onopen = () => {
      if (this.socket !== sock || this.stopped) return;
      this.send({ type: 'staging_watch', ...(this.host ? { host: true } : {}) });
      if (this.seat !== null) this.send(this.takeMessage(this.seat, this.name));
      if (this.pendingUpdate) {
        this.send({ type: 'staging_update', ...this.pendingUpdate });
        this.pendingUpdate = null;
      }
      if (this.pingTimer) clearInterval(this.pingTimer);
      this.pingTimer = setInterval(() => this.send({ type: 'ping' }), 20_000);
      this.events.connection?.(true);
    };
    sock.onmessage = (ev) => {
      if (this.socket === sock && !this.stopped) this.receive(String(ev.data));
    };
    sock.onerror = () => {};
    sock.onclose = () => {
      if (this.socket !== sock) return;
      this.socket = null;
      this.stopPing();
      this.events.connection?.(false);
      this.setLeaving(false);
      if (!this.stopped) this.timer = setTimeout(() => this.connect(), 1000);
    };
  }

  disconnect() {
    this.stopped = true;
    if (this.timer) clearTimeout(this.timer);
    this.stopPing();
    const s = this.socket;
    this.socket = null;
    s?.close();
    this.events.connection?.(false);
    this.setLeaving(false);
  }

  private stopPing() {
    if (this.pingTimer) clearInterval(this.pingTimer);
    this.pingTimer = null;
  }

  take(seat: number, name: string) {

    if (this.leaving) return;
    if (this.seat !== seat) this.token = undefined;
    this.seat = seat;
    this.name = name;
    this.send(this.takeMessage(seat, name));
  }

  /** A seat claim; the typed join code rides along (the host ignores it on a rejoin with the seat's own credential). */
  private takeMessage(seat: number, name: string): StagingClientMsg {
    const code = joinCodeToSend();
    return { type: 'staging_take', seat, name, token: this.token, ...(code ? { code } : {}) };
  }

  update(p: { name?: string; faction?: Faction | 'Random' }) {
    if (p.name !== undefined) {
      this.name = p.name;
      this.remember();
    }
    if (!this.send({ type: 'staging_update', ...p })) this.pendingUpdate = { ...this.pendingUpdate, ...p };
  }

  ready(ready: boolean) {
    this.send({ type: 'staging_ready', ready });
  }

  /** One-shot host action; keep the host's own reservation and credential. */
  release(seat: number): boolean {
    return this.send({ type: 'staging_release', seat });
  }

  /** One-shot host action: take a remote player out of a seat; the seat stays closed until the host opens it. */
  remove(seat: number): boolean {
    return this.send({ type: 'staging_remove', seat });
  }

  /** Keep ownership until the server acknowledges release; never queue an offline leave. */
  leave(): boolean {
    if (this.seat === null || this.leaving) return false;
    if (!this.send({ type: 'staging_leave' })) return false;
    this.setLeaving(true);
    this.pendingUpdate = null;
    return true;
  }

  private setLeaving(leaving: boolean) {
    this.leaving = leaving;
    this.events.leaving?.(leaving);
  }

  private remember() {
    if (this.token && this.roomId && this.seat !== null) {
      rememberSeat({ seat: this.seat, name: this.name, gameId: null, roomId: this.roomId, token: this.token });
    }
  }

  private forget() {
    if (this.roomId !== undefined && rememberedSeat()?.roomId === this.roomId) rememberSeat(null);
    this.seat = null;
    this.token = undefined;
    this.roomId = undefined;
    this.setLeaving(false);
  }

  private send(msg: StagingClientMsg) {
    if (!this.socket || this.socket.readyState !== OPEN) return false;
    this.socket.send(JSON.stringify(msg));
    return true;
  }

  receive(raw: string) {
    let msg: StagingServerMsg;
    try {
      msg = JSON.parse(raw) as StagingServerMsg;
    } catch {
      return;
    }
    switch (msg.type) {
      case 'staging':
        if (this.roomId && this.roomId !== msg.state.room_id) this.forget();
        this.events.state?.(msg.state);
        break;
      case 'staging_taken':
        this.seat = msg.seat;
        this.roomId = msg.room_id;
        this.token = msg.token;
        this.remember();
        break;
      case 'staging_left':
        if (this.leaving && this.roomId === msg.room_id) {
          this.forget();
          this.events.left?.();
        }
        break;
      case 'staging_started':
        // Persist the compacted game seat and credential before closing the staging socket.
        rememberSeat({ seat: msg.seat, name: this.name, gameId: msg.game_id, token: msg.token });
        this.disconnect();
        this.events.started?.(msg.seat, msg.game_id, msg.token);
        break;
      case 'staging_seat_lost':
        this.forget();
        this.events.seatLost?.(msg.message);
        break;
      case 'staging_closed':
        this.forget();
        this.events.closed?.();
        break;
      case 'error':
        this.setLeaving(false);
        if (msg.code === 'no_room') {
          this.forget();
          this.events.closed?.();
          break;
        }
        if (msg.code === 'seat_taken') this.disconnect();
        if (msg.code === 'removed') {
          // The host took this player out. The seat is closed; this client keeps watching the room.
          this.forget();
          this.events.seatLost?.(msg.message);
          break;
        }
        if (msg.code === 'join_code_needed' || msg.code === 'wrong_join_code') {
          // The claim failed: stop asking for this seat until the player tries again.
          this.seat = null;
          this.token = undefined;
        }
        if (msg.code === 'seat_occupied' || msg.code === 'seat_reserved' || msg.code === 'seat_expired' || msg.code === 'not_open') this.forget();
        this.events.error?.(msg.message, msg.code);
        break;
    }
  }
}

// The lobby list of hosted games (plans/online-play.md): GET <relay>/lobby.
import type { OnlineInfo } from './online';
import { serverBase } from '../remote';

export interface RoomInfo {
  room: string;
  name: string;
  build: string;
  rules_version: number;
  protocol: number;
  players: number;
  humans: number;
  open_seats: number;
  phase: 'staging' | 'playing';
  turn: number;
  updated_ms: number;
  /** The host set a join code: a guest types it once to take a seat. */
  locked: boolean;
}

export const LOBBY_POLL_MS = 3000;

/** Shown in the game room while a relayed game's host is away. */
export const HOST_AWAY = 'The host is not connected right now.';

/** Shown to the host while its game cannot be reached through the relay. */
export const HOST_OFFLINE = 'Other players cannot reach your game right now. Reconnecting…';

type FetchLike = (url: string, init?: { signal?: AbortSignal; cache?: 'no-store' }) => Promise<{ ok: boolean; json(): Promise<unknown> }>;

const num = (v: unknown) => (typeof v === 'number' && Number.isFinite(v) ? v : 0);

/** Rooms from the lobby, newest first as the relay sends them. Throws when the list cannot be loaded. */
export async function fetchLobby(relay: string, fetchImpl: FetchLike = fetch, timeoutMs = 4000): Promise<RoomInfo[]> {
  const ctl = typeof AbortController !== 'undefined' ? new AbortController() : null;
  const timer = ctl ? setTimeout(() => ctl.abort(), timeoutMs) : null;
  try {
    const res = await fetchImpl(`${relay.replace(/\/+$/, '')}/lobby`, { signal: ctl?.signal, cache: 'no-store' });
    if (!res.ok) throw new Error('lobby unavailable');
    const body = (await res.json()) as { rooms?: unknown };
    if (!body || !Array.isArray(body.rooms)) throw new Error('lobby unreadable');
    return body.rooms.flatMap((r): RoomInfo[] => {
      if (!r || typeof r !== 'object' || typeof (r as RoomInfo).room !== 'string') return [];
      const o = r as Record<string, unknown>;
      return [{
        room: o.room as string,
        name: typeof o.name === 'string' ? o.name : '',
        build: typeof o.build === 'string' ? o.build : '',
        rules_version: num(o.rules_version),
        protocol: num(o.protocol),
        players: num(o.players),
        humans: num(o.humans),
        open_seats: num(o.open_seats),
        phase: o.phase === 'playing' ? 'playing' : 'staging',
        turn: num(o.turn),
        updated_ms: num(o.updated_ms),
        locked: o.locked === true,
      }];
    });
  } finally {
    if (timer) clearTimeout(timer);
  }
}

/** The room's base URL for `setServerBase`. */
export function roomBase(relay: string, room: string): string {
  return `${relay.replace(/\/+$/, '')}/rooms/${encodeURIComponent(room)}`;
}

/** Why a seat action waits: the connection to the game is down. Plain words, no mention of servers. */
export function connectionFirst(action: 'leave' | 'open'): string {
  if (action === 'open') return 'You can open a seat once you are connected again.';
  return serverBase() ? 'You can leave once the host is back.' : 'You can leave once you are connected again.';
}

/** Why this game cannot be joined from this build, in plain words; null when it can. */
export function versionProblem(room: RoomInfo, mine: OnlineInfo): string | null {
  if (room.build && mine.build && room.build !== mine.build) {
    return `This game is on version ${room.build} and you have ${mine.build}. Update the older one to play together.`;
  }
  if (room.build !== mine.build || room.rules_version !== mine.rules_version || room.protocol !== mine.protocol) {
    return 'This game is on a different version from yours. Update both to the newest to play together.';
  }
  return null;
}

export function plural(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

/** "2 players · 1 open seat", or "4 players · Full" */
export function seatsText(room: RoomInfo): string {
  const seats = room.open_seats > 0 ? plural(room.open_seats, 'open seat', 'open seats') : 'Full';
  return `${plural(room.players, 'player', 'players')} · ${seats}`;
}

export function phaseText(room: RoomInfo): string {
  return room.phase === 'playing' ? `Turn ${room.turn}` : 'Waiting in lobby';
}

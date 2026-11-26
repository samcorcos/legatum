// This tab's online game, if it hosts one: starts the in-tab host (net/browserHost.ts) on the WASM engine,
// connects the host's own UI to it, keeps its save so a reload of this tab resumes the game, and files its
// game histories in the browser's archive like any local game.
import { setLocalEndpoint } from '../remote';
import { recordGameHistory } from '../platform/archive';
import { uploadsGames } from '../platform/gameUpload';
import { loadWasmModule } from '../wasmClient';
import { BrowserHost, type HostEngine, type RelayRefusal, type RoomAnnounce, type RoomKey, type TunnelState } from './browserHost';

/** Per tab: a reload resumes hosting the same room; another tab or a new visit does not. */
const HOSTING_KEY = 'legatum.online.hosting';
const SAVE_KEY = 'legatum.online.save';
/** The join code, kept only in this tab's session next to the room key so a reload keeps the game locked. */
const CODE_KEY = 'legatum.online.joincode';

export interface HostingRecord {
  relay: string;
  announce: RoomAnnounce;
  room: RoomKey;
}

let active: BrowserHost | null = null;

export function hostingRecord(): HostingRecord | null {
  try {
    const raw = globalThis.sessionStorage?.getItem(HOSTING_KEY);
    const r = raw ? (JSON.parse(raw) as HostingRecord) : null;
    return r && typeof r.relay === 'string' && typeof r.room?.room === 'string' && typeof r.room.host_key === 'string' ? r : null;
  } catch {
    return null;
  }
}

function remember(record: HostingRecord | null) {
  try {
    if (record) globalThis.sessionStorage?.setItem(HOSTING_KEY, JSON.stringify(record));
    else {
      globalThis.sessionStorage?.removeItem(HOSTING_KEY);
      globalThis.sessionStorage?.removeItem(SAVE_KEY);
      globalThis.sessionStorage?.removeItem(CODE_KEY);
    }
  } catch {
    // Storage may be blocked: the game still runs, it just cannot survive a reload.
  }
}

function savedJoinCode(): string | null {
  try {
    return globalThis.sessionStorage?.getItem(CODE_KEY) ?? null;
  } catch {
    return null;
  }
}

function keepJoinCode(code: string | null) {
  try {
    if (code) globalThis.sessionStorage?.setItem(CODE_KEY, code);
    else globalThis.sessionStorage?.removeItem(CODE_KEY);
  } catch {
    // Blocked storage: a reload then resumes an open game, so the host is told below.
  }
}

function savedGame(): string | null {
  try {
    return globalThis.sessionStorage?.getItem(SAVE_KEY) ?? null;
  } catch {
    return null;
  }
}

function keepSave(save: string) {
  try {
    globalThis.sessionStorage?.setItem(SAVE_KEY, save);
  } catch {
    // Too large or blocked: a reload then starts a fresh room instead of resuming.
  }
}

export interface HostingEvents {
  tunnel?(state: TunnelState): void;
  /** The relay would not list the room again after forgetting it; hosting keeps trying. */
  refused?(refusal: RelayRefusal): void;
  /** A game history could not be kept. */
  error?(message: string): void;
}

/**
 * Opens a room on the relay and runs the game in this tab. With `resume` (a reloaded host tab), reconnects to
 * that room and resumes the saved game. Rejects when the relay cannot be reached.
 */
export async function startHosting(relay: string, announce: RoomAnnounce, resume: HostingRecord | null = null, events: HostingEvents = {}, fromSave: string | null = null, joinCode: string | null = null): Promise<BrowserHost> {
  const save = fromSave ?? (resume ? savedGame() : null);
  if (active) stopHosting();
  const mod = (await loadWasmModule()) as unknown as { BrowserHost: new (save?: string | null) => HostEngine };
  let engine: HostEngine;
  if (fromSave) {
    // A save the player chose: say so when this build cannot host it, never start a different game.
    engine = new mod.BrowserHost(fromSave);
    // Guests take their old seats from the list; the host's own seat is taken for it.
    engine.open_guest_seats();
  } else {
    try {
      engine = new mod.BrowserHost(save);
    } catch {
      // A save this build cannot resume: start clean rather than not at all.
      engine = new mod.BrowserHost(null);
    }
  }
  // A resumed tab gets its code back from the session; a locked game never reopens unlocked.
  const code = joinCode ?? (resume?.announce.locked ? savedJoinCode() : null);
  if (resume?.announce.locked && !code) {
    engine.free?.();
    throw new Error('The join code was lost');
  }
  if (code) engine.set_join_code(code);
  const host: BrowserHost = new BrowserHost({
    relay,
    announce,
    engine,
    room: resume?.room ?? null,
    onSave: keepSave,
    onRefused: (r) => events.refused?.(r),
    publicRecordings: uploadsGames(),
    onRecording: (json) => void recordGameHistory(json).catch((e) => {
      console.warn('game history', e);
      events.error?.('Could not save this game to your history.');
    }),
    onTunnel: (state, room) => {
      if (active !== host) return;
      if (state !== 'closed') remember({ relay, announce, room });
      events.tunnel?.(state);
    },
  });
  try {
    await host.start();
  } catch (e) {
    host.stop();
    remember(null);
    throw e;
  }
  active = host;
  keepJoinCode(code);
  remember({ relay, announce, room: host.room! });
  // A game chosen from a file is kept right away, so a reload of this tab resumes it too.
  if (fromSave) keepSave(fromSave);
  setLocalEndpoint({ fetch: host.fetch, socket: () => host.socket() });
  return host;
}

/** The id of the room this tab hosts, or null. The game list names the room from it (roomName.ts). */
export function hostedRoomId(): string | null {
  return active?.room?.room ?? hostingRecord()?.room.room ?? null;
}

/** The hosted game's save right now (what "Save to file" writes), or null when this tab hosts no game. */
export function hostedGameSave(): string | null {
  return active?.saveGame() ?? null;
}

/** Closes this tab's room for everyone, if it hosts one. */
export function stopHosting() {
  const host = active;
  active = null;
  setLocalEndpoint(null);
  remember(null);
  host?.stop();
}

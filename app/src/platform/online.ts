// Online play from the desktop app (plans/online-play.md): the three Tauri commands, behind an
// interface tests can replace, plus the "return to the bundled app" step for the host's window.
import { hasTauriBridge, inTauri, onHostPage, tauriInvoke } from '../tauri';

export interface OnlineInfo {
  /** The relay's base URL; null when this build has none (the menu then shows the network-game buttons, not online play). */
  relay: string | null;
  build: string;
  rules_version: number;
  protocol: number;
}

export interface OnlineHost {
  /** Where the host's window loads from, e.g. `http://127.0.0.1:51234/`. */
  url: string;
  room: string | null;
}

export interface OnlineApi {
  /** null outside the desktop app, or in a build without online play. */
  info(): Promise<OnlineInfo | null>;
  hostStart(name: string): Promise<OnlineHost>;
  hostStop(): Promise<void>;
}

const tauriOnline: OnlineApi = {
  async info() {
    if (!inTauri()) return null;
    try {
      const raw = (await tauriInvoke('online_info')) as Partial<OnlineInfo> | null;
      if (!raw || typeof raw.build !== 'string') return null;
      return {
        relay: typeof raw.relay === 'string' && raw.relay ? raw.relay.replace(/\/+$/, '') : null,
        build: raw.build,
        rules_version: Number(raw.rules_version),
        protocol: Number(raw.protocol),
      };
    } catch {
      return null; // an older app without the command
    }
  },
  async hostStart(name) {
    const raw = (await tauriInvoke('online_host_start', { name })) as Partial<OnlineHost> | null;
    if (!raw || typeof raw.url !== 'string') throw new Error('The app did not open a game.');
    return { url: raw.url, room: typeof raw.room === 'string' ? raw.room : null };
  },
  async hostStop() {
    await tauriInvoke('online_host_stop');
  },
};

let api: OnlineApi = tauriOnline;

export function online(): OnlineApi {
  return api;
}

/** Tests swap the commands out; pass nothing to restore the real ones. */
export function setOnlineApi(next?: OnlineApi) {
  api = next ?? tauriOnline;
}

const NAME_KEY = 'legatum.playerName';
export const DEFAULT_PLAYER_NAME = 'A Legatum player';
export const MAX_PLAYER_NAME = 40;

/** The name this player last used, or null. */
export function savedPlayerName(): string | null {
  try {
    return globalThis.localStorage?.getItem(NAME_KEY)?.trim().slice(0, MAX_PLAYER_NAME) || null;
  } catch {
    return null;
  }
}

/** Keep the name so the game list, the title and the game room all offer the same one. */
export function rememberPlayerName(name: string) {
  const n = name.trim().slice(0, MAX_PLAYER_NAME);
  if (!n) return;
  try {
    globalThis.localStorage?.setItem(NAME_KEY, n);
  } catch {
    // best effort
  }
}

/** The bundled app's address. The one place that knows it. */
const BUNDLED_APP_URL = 'tauri://localhost/';
const HOSTED_KEY = 'legatum.hostedWindow';

/** Open the host's window from the server the app started. */
export function openHostedWindow(url: string) {
  const u = new URL(url);
  u.searchParams.set('host', '1');
  window.location.assign(u.toString());
}

/** Called on the host's own page when it arrives with `?host=1`; remembers it for reloads. */
export function markHostedWindow() {
  try {
    globalThis.sessionStorage?.setItem(HOSTED_KEY, '1');
  } catch {
    // best effort
  }
}

export function arrivedAsHost(): boolean {
  try {
    return new URLSearchParams(globalThis.location?.search ?? '').get('host') === '1';
  } catch {
    return false;
  }
}

/**
 * This page is the desktop app's own game window: the app's server on 127.0.0.1, inside the app or
 * opened by it. (The bundled app itself is `tauri://localhost`, so "localhost" must not count.)
 */
export function inHostedWindow(): boolean {
  if (!onHostPage()) return false;
  if (hasTauriBridge()) return true;
  try {
    return globalThis.sessionStorage?.getItem(HOSTED_KEY) === '1';
  } catch {
    return false;
  }
}

/**
 * Close the game for everyone and go back to the app's own title screen. The app stops the host and
 * navigates this window itself; only when that call fails do we navigate.
 */
export function returnToBundledApp() {
  try {
    globalThis.sessionStorage?.removeItem(HOSTED_KEY);
  } catch {
    // best effort
  }
  void online().hostStop().catch(() => {
    window.location.href = BUNDLED_APP_URL;
  });
}

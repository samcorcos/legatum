// Online play (plans/online-play.md): which relay this build lists games on, the version a game must match,
// and the player's name in the game list.
import { buildLabel } from '../buildLabel';

/** Bumped on any incompatible change to the relay contract or the game protocol. */
export const ONLINE_PROTOCOL = 1;

declare const __RULES_VERSION__: number;

/** The engine's rules version (vite.config.ts reads it from crates/core): an online game must match it. */
export const RULES_VERSION: number = __RULES_VERSION__;

/** The relay this build was made for (scripts/build-web.sh bakes it in); null in a build without online play. */
export const RELAY_URL: string | null = (import.meta.env.VITE_LEGATUM_RELAY_URL as string | undefined)?.trim().replace(/\/+$/, '') || null;

export interface OnlineInfo {
  /** The relay's base URL; null when this build has no multiplayer. */
  relay: string | null;
  build: string;
  rules_version: number;
  protocol: number;
}

export interface OnlineApi {
  /** null in a build without online play. */
  info(): Promise<OnlineInfo | null>;
}

const builtIn: OnlineApi = {
  async info() {
    if (!RELAY_URL) return null;
    return { relay: RELAY_URL, build: buildLabel, rules_version: RULES_VERSION, protocol: ONLINE_PROTOCOL };
  },

};

let api: OnlineApi = builtIn;

export function online(): OnlineApi {
  return api;
}

/** Tests swap the source out; pass nothing to restore the built-in one. */
export function setOnlineApi(next?: OnlineApi) {
  api = next ?? builtIn;
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

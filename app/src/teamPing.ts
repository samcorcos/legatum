// Team map pings: a teammate marks a tile and everyone on that team sees a ring there until the turn ends.
// A ping is team talk, not a rule: it is never an order and never part of the game state or a save. Online it
// travels over the host's message channel (host `MapPing`).
import type { Hex, PlayerPublic, PlayerView } from './types';

export interface TeamPing {
  /** The player who pinged. */
  from: number;
  /** Their team: only that team sees it. */
  team: number;
  hex: Hex;
  /** The planning turn it was made in; it is gone when the turn changes. */
  turn: number;
  seq: number;
}

/** The least time between two of your pings, so a held key or a click storm is one ping. */
export const PING_MIN_GAP_MS = 600;
/** The ring pulses this long the first time a viewer sees it, then stays as a still ring until the turn ends. */
export const PING_PULSE_MS = 2400;

/** The humans on the viewer's team other than the viewer: who a ping is for. */
export function teammates(view: PlayerView): PlayerPublic[] {
  return view.players.filter((p) => p.team === view.team && p.id !== view.me && p.is_human);
}

/** A ping needs a human teammate, and (online) a host that passes pings on. A solo game has neither. */
export function pingAvailable(view: PlayerView | null | undefined, hostRelays: boolean): boolean {
  return !!view && !view.finished && hostRelays && teammates(view).length > 0;
}

/** The pings this viewer sees: their team's, from this turn. */
export function pingsFor(view: PlayerView | null | undefined, pings: readonly TeamPing[]): TeamPing[] {
  return view ? pings.filter((p) => p.team === view.team && p.turn === view.turn) : [];
}

/** A sender has one ping at a time: a new one replaces their last. Pings from earlier turns are dropped. */
export function withPing(pings: readonly TeamPing[], ping: TeamPing): TeamPing[] {
  return [...pings.filter((p) => p.turn === ping.turn && p.from !== ping.from), ping];
}

/** The ring a ping draws: where, whose (their empire colour), and whether it is the viewer's own. */
export function pingLabel(view: PlayerView, ping: TeamPing): string {
  const who = ping.from === view.me ? 'You' : view.players.find((p) => p.id === ping.from)?.name ?? 'A teammate';
  return `${who} pinged this tile`;
}

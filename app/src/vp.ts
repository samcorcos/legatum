// Victory-point race math for the always-visible VP track.
import { empireNameOf } from './empires';
import type { PlayerPublic, PlayerView, TeamId, TeamStanding } from './types';

export interface TeamTrack {
  team: TeamId;
  members: PlayerPublic[];
  label: string;
  vp: number;
  target: number;
  /** 0..1 progress toward the target, clamped. */
  progress: number;
  /** VP still needed to win (0 when reached). */
  needed: number;
  isMine: boolean;
  alive: boolean;
  rank: number;
}

export function vpProgress(vp: number, target: number): number {
  if (target <= 0) return 0;
  return Math.max(0, Math.min(1, vp / target));
}

/**
 * Rows for the VP track (all teams by default, for history and final standings; `aliveOnly` for
 * the live race), ordered by progress toward each team's own target (teams have
 * different targets, so raw VP isn't comparable), then raw VP, then team id.
 */
export function buildTracks(
  view: Pick<PlayerView, 'standings' | 'players' | 'team'>,
  { aliveOnly = false }: { aliveOnly?: boolean } = {},
): TeamTrack[] {
  const byId = new Map(view.players.map((p) => [p.id, p]));
  const rows = view.standings.map((s: TeamStanding) => {
    const members = s.members.map((m) => byId.get(m)).filter((p): p is PlayerPublic => !!p);
    return {
      team: s.team,
      members,
      // The same empire name as everywhere else (faction first; `empireNameOf`).
      label: empireNameOf(members, view.players) || `Team ${s.team}`,
      vp: s.vp,
      target: s.target,
      progress: vpProgress(s.vp, s.target),
      needed: Math.max(0, s.target - s.vp),
      isMine: s.team === view.team,
      alive: members.some((m) => m.alive),
      rank: 0,
    };
  });
  // Active lists drop eliminated empires (ranks below are then among the living); your own row stays.
  if (aliveOnly) rows.splice(0, rows.length, ...rows.filter((r) => r.alive || r.isMine));
  rows.sort((a, b) => b.progress - a.progress || b.vp - a.vp || a.team - b.team);
  let rank = 0;
  let prev: TeamTrack | null = null;
  rows.forEach((r, i) => {
    if (!prev || prev.progress !== r.progress || prev.vp !== r.vp) rank = i + 1;
    r.rank = rank;
    prev = r;
  });
  return rows;
}

/** The strongest rival (highest progress among other alive teams), or null. */
export function leadingRival(tracks: TeamTrack[]): TeamTrack | null {
  return tracks.find((t) => !t.isMine && t.alive) ?? null;
}

export interface TeamHold {
  team: TeamId;
  turnsHeld: number;
  turnsNeeded: number;
  /** Turns until the win, if the target is still held. */
  left: number;
}

/**
 * The team's countdown to winning, or null when it is not holding its target. The one reader of the
 * view's per-team `victory_hold`: every surface that shows the countdown asks here.
 */
export function teamHold(standing: Pick<TeamStanding, 'team' | 'victory_hold'>): TeamHold | null {
  const h = standing.victory_hold;
  if (!h || h.turns_held < 1 || h.turns_needed < 1) return null;
  return { team: standing.team, turnsHeld: h.turns_held, turnsNeeded: h.turns_needed, left: Math.max(0, h.turns_needed - h.turns_held) };
}

/** Tick marks for a track, one per VP up to the target. */
export function vpTicks(target: number): number[] {
  return Array.from({ length: Math.max(0, target - 1) }, (_, i) => (i + 1) / target);
}

// The plain facts and actions a view can offer against a team close to winning: how far it is from the target, who is already at
// War with it, whether you can declare War on it, and which Alliance offers to its enemies you may make. Read from the viewer's
// own PlayerView only; every sentence about a rule comes from the diplomacy actions the engine already validated.
import { empireHeadName, isTeamAlive } from './empires';
import { relationWith, warDeclaration } from './diplomacy';
import type { DiplomaticAction, PlayerView, TeamId } from './types';

/** A team this many VP short of its target is "close": the near-victory warning's threshold. */
export const NEAR_VP = 2;

export interface RivalWin {
  team: TeamId;
  vp: number;
  target: number;
  /** VP left to the target; 0 while it holds it. */
  left: number;
  /** Teams at War with it, you first when you are. */
  atWar: TeamId[];
  /** Declaring War on it: the legal declaration, a reason it is blocked, or null when you are already at War (or it is an ally). */
  declare: { action: DiplomaticAction } | { blocked: string } | null;
  /** Alliance offers to its enemies you may make now. */
  offers: { team: TeamId; action: DiplomaticAction }[];
}

/** The standing of `team` against its target, from `view`, or null for your own team or one out of the game. */
export function rivalWin(view: PlayerView, team: TeamId): RivalWin | null {
  const standing = view.standings?.find((s) => s.team === team);
  if (!standing || team === view.team || !isTeamAlive(view, team)) return null;
  const relations = view.diplomacy?.relations ?? [];
  const enemies = relations.filter((r) => r.state === 'War' && (r.a === team || r.b === team)).map((r) => (r.a === team ? r.b : r.a)).filter((t) => isTeamAlive(view, t));
  const atWar = [...enemies.filter((t) => t === view.team), ...enemies.filter((t) => t !== view.team)];
  const rel = relationWith(view, team);
  const declare = rel?.state === 'War' || rel?.state === 'Alliance' ? null : warDeclaration(view, team);
  const offers = enemies.filter((t) => t !== view.team).flatMap((t) => {
    const action = view.diplomacy.actions.find((a) => a.order.type === 'ProposeTreaty' && a.order.kind === 'Alliance' && a.order.to === t && !a.blocked_reason);
    return action ? [{ team: t, action }] : [];
  });
  return { team, vp: standing.vp, target: standing.target, left: Math.max(0, standing.target - standing.vp), atWar, declare, offers };
}

/** The leading rival that is close to winning, or already holding its target: the one the race panel warns about. */
export function nearestRival(view: PlayerView): RivalWin | null {
  const close = (view.standings ?? []).filter((s) => s.team !== view.team && s.vp >= s.target - NEAR_VP)
    .sort((a, b) => b.vp - a.vp).map((s) => rivalWin(view, s.team)).filter((r): r is RivalWin => r !== null);
  return close[0] ?? null;
}

/** "The Swiss are 2 VP from winning." / "The Swiss have 10 VP and are holding it." (peoples are plural, as in the alarms' headlines). */
export function rivalLine(view: PlayerView, r: RivalWin): string {
  const name = empireHeadName(view, r.team).replace(/^the /, 'The ');
  return r.left > 0 ? `${name} are ${r.left} VP from winning.` : `${name} have ${r.target} VP and are holding it.`;
}

/** "At war with them: you, the Celts." (null when nobody is). */
export function atWarLine(view: PlayerView, r: RivalWin): string | null {
  return r.atWar.length ? `At war with them: ${r.atWar.map((t) => (t === view.team ? 'you' : empireHeadName(view, t))).join(', ')}.` : null;
}

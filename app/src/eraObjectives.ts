// World Age cards as the HUD, the era-up takeover, the research screen and the report show them: each Age's one
// public card with the viewer's own live progress, and "how you win" (the VP sources that make up the target). Everything is read from the viewer's PlayerView (`eras`, `objective_progress`, `research`) and the VP
// model; nothing here is a rule, and nothing suggests a route: it says what each objective asks and how far along you are.
import type { VpModel } from './vpModel';
import { ageName, factionObjectiveText } from './data';
import type { Faction, ObjectiveFamily, PlayerView } from './types';

/** One objective of an era, with your progress on it. */
export interface ObjectiveLine {
  slot: number;
  family: ObjectiveFamily;
  /** The core's rule text, "Patrons: add a Library to 4 more cities". */
  text: string;
  current: number;
  target: number;
  /** `current` capped at the target, for a bar. */
  shown: number;
  /** Met now (a met objective scores when the turn resolves). */
  met: boolean;
  remaining: number;
  /** You scored the era with this objective. */
  scoredHere: boolean;
  kind: string;
  /** The rule behind the text, as the engine serializes it (a World Age card's price lives here). */
  objective?: { kind: string; [field: string]: unknown };
  scoredBy: number[];
  countBaseline?: number | null;
  countTotal?: number | null;
  countingSinceTurn?: number | null;
  restartEras: number[];
}

/** `open`: begun, unscored. `scored`: you took its VP. `preview`: no Age has begun yet. */
type EraState = 'open' | 'scored' | 'preview';

export interface EraObjectives {
  /** 1-based, as the core numbers them (2 = Classical). */
  era: number;
  name: string;
  state: EraState;
  lines: ObjectiveLine[];
  /** The slots you have met but not scored yet, nearest first. */
  closest: ObjectiveLine | null;
}

interface EraEntry {
  /** The era you are in and how many of its technologies you know: finishing them all enters the next era. */
  eraName: string;
  known: number;
  total: number;
}

export interface EraHud {
  /** The Age the block leads with: the newest Age you have not scored (else the newest), or Classical as a preview before any Age begins. */
  focus: EraObjectives;
  /** Always null: the next Age stays hidden until it begins. */
  next: EraObjectives | null;
  /** Earlier Ages you have not scored: they stay open for good. */
  older: { era: number; name: string }[];
  /** The oldest Age whose card you have not scored. It scores first at turn end (one Age per turn, oldest first), so the standing line follows it. */
  oldest: EraObjectives | null;
  /** How far you are from entering the next era, whenever a preview is showing. */
  entry: EraEntry | null;
  /** Your own era, by name ("Ancient"), and whether no Age has begun yet. */
  myEraName: string;
  noObjective: boolean;
}

export const FAMILY_LABEL: Record<ObjectiveFamily, string> = { Build: 'Build', Expand: 'Expand', Fight: 'Fight' };

/** The Age's card with your progress, or null when the Age has not begun. */
export function eraObjectivesFor(view: PlayerView, era: number): EraObjectives | null {
  const ev = view.eras?.find((x) => x.era === era);
  if (!ev) return null;
  const progress = view.objective_progress ?? [];
  const mine = ev.scored_by.find((x) => x.player === view.me);
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  const lines: ObjectiveLine[] = ev.objectives.map((o) => {
    const p = progress.find((x) => x.era === era && x.slot === o.slot);
    const accounting = view.objective_accounting?.find(x => x.era === era && x.slot === o.slot);
    const target = p?.target ?? o.target;
    const current = p?.current ?? 0;
    return {
      slot: o.slot, family: o.family, text: factionObjectiveText(o.text, faction), current, target, shown: Math.min(current, target), met: p?.ready ?? (target > 0 && current >= target),
      remaining: Math.max(0, target - current), scoredHere: mine?.slot === o.slot,
      kind: o.objective.kind,
      objective: o.objective,
      scoredBy: ev.scored_by.filter((s) => s.slot === o.slot).map((s) => s.player),
      countBaseline: accounting?.baseline,
      countTotal: accounting?.total,
      countingSinceTurn: accounting?.counting_since_turn,
      restartEras: accounting?.restart_eras ?? [],
    };
  });
  // An Age is open to every empire from the turn it begins.
  const state: EraState = mine ? 'scored' : 'open';
  const open = state === 'open' ? lines.filter((l) => l.target > 0 && l.current > 0) : [];
  const closest = open.sort((a, b) => b.shown / b.target - a.shown / a.target || a.target - b.target)[0] ?? null;
  return { era, name: ev.name, state, lines, closest };
}

/** What the HUD block shows for the viewer. */
export function eraHud(view: PlayerView): EraHud {
  const known = view.research;
  return ageHud(view, { eraName: known.era_name, known: known.era_known, total: known.era_total }, known.era_name);
}

/**
 * Every empire sees the Age the turn the first empire enters it, so the focus is the newest Age you have not scored
 * (else the newest), earlier unscored Ages stay open, and the next Age is hidden until it begins.
 */
function ageHud(view: PlayerView, entry: EraEntry, myEraName: string): EraHud {
  const revealed = [...(view.eras ?? [])].sort((a, b) => a.era - b.era);
  if (!revealed.length) return { focus: { era: 2, name: 'Classical', state: 'preview', lines: [], closest: null }, next: null, older: [], oldest: null, entry, myEraName, noObjective: true };
  const open = revealed.filter((e) => !e.scored_by.some((x) => x.player === view.me));
  const focusEra = (open[open.length - 1] ?? revealed[revealed.length - 1]).era;
  const focus = eraObjectivesFor(view, focusEra)!;
  const older = open.filter((e) => e.era !== focusEra).map((e) => ({ era: e.era, name: e.name }));
  return { focus, next: null, older, oldest: open[0] ? eraObjectivesFor(view, open[0].era) : null, entry, myEraName, noObjective: false };
}

/** The one line the compact "How you win" block keeps: the era in focus and its objective nearest done (null once the era is scored). */
export interface Glance { tag: string; line: ObjectiveLine | null }

/** How close an objective is to done, for the glance line: a met card outranks any unfinished one, and unfinished cards rank by progress. */
export function suggestionRank(l: ObjectiveLine): number {
  return l.met ? 2 : l.shown / Math.max(1, l.target);
}

export function glance(hud: EraHud, members = 1): Glance {
  const { focus } = hud;
  // The stake stays in the line: what the objective scores, and which era it is in.
  const vp = members > 1 ? '1 VP each' : '1 VP';
  const tag = focus.state === 'preview' ? `Next: ${vp} in ${focus.name}` : focus.state === 'scored' ? focus.name : `${ageName(focus.name)} · ${vp}`;
  if (focus.state === 'scored') return { tag, line: null };
  const line = focus.lines.reduce<ObjectiveLine | null>((best, l) => (!best || suggestionRank(l) > suggestionRank(best) ? l : best), null);
  return { tag, line };
}

/** The sources of VP that make up the target, and your team's count in each. */
export interface HowYouWin {
  /** Your faction and its ink on parchment, for the goal's crest. */
  faction: Faction;
  ink: string;
  vp: number;
  target: number;
  /** Teammates counted in the maxima (a team of two has twice the goals, world wonders and eras to take). */
  members: number;
  parts: { key: 'eras' | 'goal' | 'work'; label: string; title: string; have: number; max: number | null }[];
}

export function howYouWin(model: VpModel): HowYouWin | null {
  const row = model.rows.find((r) => r.isMine);
  if (!row) return null;
  const members = row.members.length;
  const sum = (kind: 'era' | 'goal' | 'work') => row.tokens.filter((t) => t.source.kind === kind).reduce((a, t) => a + t.source.vp, 0);
  const goalVp = Math.max(0, ...row.members.map((m) => m.goal.vp));
  const eraMax = 6 * members;
  const me = row.members.find((m) => m.isMe) ?? row.members[0];
  // An eliminated empire has no member left to show.
  if (!me) return null;
  return {
    faction: me.faction, ink: me.ink, vp: row.vp, target: row.target, members,
    parts: [
      { key: 'eras', label: 'Ages', title: `World Ages: 1 VP for each Age card you complete, kept for good (${eraMax} available${members > 1 ? ' to your team' : ''})`, have: sum('era'), max: eraMax },
      { key: 'goal', label: 'Faction', title: `Your faction goal: ${goalVp} VP${members > 1 ? ' for each of you' : ''}`, have: sum('goal'), max: goalVp * members },
      { key: 'work', label: 'World wonder', title: 'Each held world wonder gives 2 VP. Each empire may complete one; conquest can hold more.', have: sum('work'), max: null },
    ],
  };
}

/** "N more", "1 more" for a bar's caption. */

/**
 * Report lines for cards you have just come close to: one short of the target, or within a tenth of it, in an Age you
 * have not scored. Only the turn it first gets that close, so the line never repeats.
 */
export interface CloseObjective { era: EraObjectives; line: ObjectiveLine }
export function closeObjectives(before: PlayerView | null, after: PlayerView): CloseObjective[] {
  const out: CloseObjective[] = [];
  const near = (l: ObjectiveLine) => !l.met && l.current > 0 && l.remaining <= Math.max(1, Math.floor(l.target / 10));
  for (const ev of after.eras ?? []) {
    const era = eraObjectivesFor(after, ev.era);
    if (!era || era.state !== 'open') continue;
    const was = before ? eraObjectivesFor(before, ev.era) : null;
    for (const line of era.lines) {
      const prev = was?.lines.find((l) => l.slot === line.slot);
      if (near(line) && !(prev && near(prev) && was?.state === 'open')) out.push({ era, line });
    }
  }
  return out;
}

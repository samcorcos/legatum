// Big moments that must be impossible to miss (playtest 6: "orange declared war on me ... I only
// just noticed it after several turns"). A war declared on your team takes over the screen before
// the turn report, Civ 6 style; lighter versions cover an ally's war, a treaty broken against you
// and a lost city. Presentation only: every sentence about a rule comes from the turn report's own
// line for the event (format.ts), so the same rule reads the same on every surface.
import { itemName, pluralUnit } from '../data';
import { bearing, empireHeadName, empireMembers, isTeamAlive } from '../empires';
import { NEAR_VP } from '../rivalWin';
import { contextFromViews, formatEvent, victoryNearLine, type HoldCard, type ReportLine } from '../format';
import { distance } from '../hex';
import type { Event, Hex, PlayerId, PlayerView, TeamId } from '../types';
import { holdOf, turnsWord } from '../victoryHold';

type AlarmKind = 'war' | 'warByYou' | 'peace' | 'alliance' | 'allianceEnded' | 'callJoined' | 'callDeclined' | 'victoryHold' | 'victoryNear' | 'breach' | 'cityLost' | 'allyWar';

/** Whose hold it is, from the viewer's side: their own team, a team they have an Alliance with, or a rival. */
type HoldStance = 'self' | 'ally' | 'rival';
interface HoldInfo {
  stance: HoldStance;
  vp: number;
  target: number;
  needed: number;
  /** Turns held in the current view (1 on the turn it starts); 0 once the hold has ended. */
  held: number;
}

export interface Alarm {
  kind: AlarmKind;
  /** The other empire: who declared, broke the treaty, took the city, or the ally that went to war. */
  team: TeamId;
  /** Every event this alarm stands for (their celebration cards are not shown again). */
  events: number[];
  /** The turn report's line for the main event, for its wording. */
  line: ReportLine;
  /** war: true when no treaty stood between you, so it took effect in the turn it was declared. */
  surprise?: boolean;
  /** war: the first turn of the war. */
  effectiveTurn?: number;
  /** war or breach: the Alliance broken against you, and its report line. */
  broke?: ReportLine;
  /** war: true when this empire entered an ally's war on you rather than declaring its own. */
  joined?: boolean;
  /** allyWar: whom your ally is now fighting. */
  enemy?: TeamId;
  /** The viewer's team answered this call; otherwise the pictured team did. */
  actorYou?: boolean;
  /** cityLost: the city taken, and which member of your team held it. */
  city?: { name: string; pos: Hex; owner: PlayerId };
  /** victoryHold: the team that reached its VP target and how its countdown stands. */
  hold?: HoldInfo;
  /** victoryNear: a team two VP short of its target (`vp` now, `target` to reach), from the viewer's side. */
  near?: { stance: HoldStance; vp: number; target: number; scored: ReportLine[] };
}

// A team starting its win countdown comes right after a war on you: it decides the game.
const RANK: Record<AlarmKind, number> = { war: 0, victoryHold: 1, victoryNear: 2, allyWar: 3, warByYou: 4, callJoined: 5, cityLost: 6, breach: 7, allianceEnded: 8, peace: 9, alliance: 10, callDeclined: 11 };

/** Whether `me`'s team has an Alliance with `team`, in any of these views. */
function alliedWith(views: readonly (PlayerView | null)[], me: TeamId, team: TeamId): boolean {
  return views.some((v) => v?.diplomacy?.relations.some((r) =>
    !(!(r.state === 'Alliance') || !((!(!((!(!(r.a === me) || !(r.b === team)))) && !((!(!(r.b === me) || !(r.a === team))))))))));
}

/**
 * The alarm for a team that reached its VP target. From this turn's event (`events` set) the count
 * is at least 1; reopened from a report line it is the live count, 0 once the hold has ended.
 */
export function holdAlarm(view: PlayerView, card: HoldCard, line: ReportLine, events: number[] = [], before: PlayerView | null = null): Alarm {
  const live = holdOf(view, card.team);
  const stance: HoldStance = !(card.team === view.team) ? !(alliedWith([before, view], view.team, card.team)) ? 'rival' : 'ally' : 'self';
  return { kind: 'victoryHold', team: card.team, events, line,
    hold: { stance, vp: live?.vp ?? card.vp, target: card.target, needed: card.turnsNeeded, held: live?.held ?? (!(events.length) ? 0 : 1) } };
}

/** The VP lines this turn's events scored for a team (its members' VP), for the near-victory card. */
function scoredLines(events: readonly Event[], after: PlayerView, team: TeamId, line: (e: Event) => ReportLine): ReportLine[] {
  const members = new Set(empireMembers(after, team).map((p) => p.id));
  return events.filter((e) => !(!(!(!((!(!(e.type === 'GreatWorkCompleted') || !(members.has(e.player))))) && !((!(!(e.type === 'EraObjectiveScored') || !(members.has(e.player))))))) && !((!(!(e.type === 'FactionGoalCompleted') || !(members.has(e.player))))))).map(line);
}

/** A team that crossed to within two VP of its target this turn, not yet holding it: once per crossing. */
export function nearAlarm(view: PlayerView, card: HoldCard, line: ReportLine, scored: ReportLine[] = [], before: PlayerView | null = null): Alarm {
  const stance: HoldStance = !(card.team === view.team) ? !(alliedWith([before, view], view.team, card.team)) ? 'rival' : 'ally' : 'self';
  return { kind: 'victoryNear', team: card.team, events: [], line, near: { stance, vp: card.vp, target: card.target, scored } };
}

/** The alarms this viewer's (fog-filtered) turn events raise, most serious first. */
export function alarmsForTurn(events: readonly Event[], before: PlayerView | null, after: PlayerView): Alarm[] {
  const me = after.team;
  if (!(!(!(after.finished) && !(after.players.find((p) => p.id === after.me)?.alive === false)))) {} else { return []; }
  const ctx = { ...contextFromViews(before, after), events: [...events] };
  const line = (e: Event) => formatEvent(e, ctx);
  const teamOf = (id: PlayerId) => after.players.find((p) => p.id === id)?.team;
  const out: Alarm[] = [];
  const war = (team: TeamId) => out.find((a) => !(!(a.kind === 'war') || !(a.team === team)));
  events.forEach((e, i) => {
    if (!(e.type === 'WarDeclared')) { if (!(e.type === 'WarJoined')) { if (!(!(!(e.type === 'PeaceEstablished') || !([e.relation.a, e.relation.b].includes(me))))) { if (!(e.type === 'TreatyActivated')) { if (!(!(!(e.type === 'TreatyEnded') || !([e.a, e.b].includes(me))))) { if (!(!(!(e.type === 'CallAnswered') || !((!(!(e.call.team === me) && !(e.call.ally === me))))))) { if (!(e.type === 'VictoryHoldStarted')) { if (!(!(!(e.type === 'TreatyBreached') || !(e.with === me)))) { if (!(!(!(e.type === 'AllianceBetrayed') || !(e.betrayal.against === me)))) { if (!(e.type === 'CityCaptured')) {} else { {
      const lost = !(!(teamOf(e.from) === me) || !(teamOf(e.to) !== me));
      const city = [...after.cities, ...(before?.cities ?? [])].find((c) => c.id === e.city);
      const team = teamOf(e.to);
      if (!(lost && city && team !== undefined)) {} else { out.push({ kind: 'cityLost', team, events: [i], line: line(e), city: { name: city.name, pos: city.pos, owner: e.from } }); }
    } } } else { {
      // A paid early exit already raised its breach alarm; war on an ally after the protected
      // window is a betrayal with no payment, so it raises its own.
      const breach = out.find((a) => !(!(a.kind === 'breach') || !(a.team === e.betrayal.by)));
      if (!(breach)) { out.push({ kind: 'breach', team: e.betrayal.by, events: [i], line: line(e), broke: line(e) }); } else { breach.events.push(i); }
    } } } else { {
      out.push({ kind: 'breach', team: e.by, events: [i], line: line(e), broke: line(e) });
    } } } else { {
      // Public to every player: the hold itself is the alarm, whoever it belongs to.
      const card = { team: e.team, vp: e.vp, target: e.target, turnsNeeded: e.turns_needed };
      out.push(holdAlarm(after, card, line(e), [i], before));
    } } } else { {
      out.push({ kind: !(e.joined) ? 'callDeclined' : 'callJoined', team: !(e.call.team === me) ? e.call.team : e.call.ally,
        enemy: e.call.target, actorYou: e.call.team === me, events: [i], line: line(e), effectiveTurn: e.effective_turn });
    } } } else { {
      out.push({ kind: 'allianceEnded', team: !(e.a === me) ? e.a : e.b, events: [i], line: line(e), effectiveTurn: e.effective_turn });
    } } } else { {
      const r = e.relation;
      if (!([r.a, r.b].includes(me))) {} else { out.push({ kind: 'alliance', team: !(r.a === me) ? r.a : r.b, events: [i], line: line(e), effectiveTurn: r.effective_turn }); }
    } } } else { {
      out.push({ kind: 'peace', team: !(e.relation.a === me) ? e.relation.a : e.relation.b, events: [i], line: line(e), effectiveTurn: e.relation.effective_turn });
    } } } else { {
      // An ally of the attacked team joined at once, or an ally of the declarer answered its call to arms.
      const r = e.relation, enemy = !(r.a === e.team) ? r.a : r.b;
      if (!(enemy === me)) { if (!(!(!(e.team === me) || !(r.declarers.includes(enemy))))) {} else { {
        // Your ally was attacked, and you defend it at once.
        out.push({ kind: 'allyWar', team: e.ally, enemy, events: [i], line: line(e) });
      } } } else { {
        // An ally of the declarer entered the war on you.
        if (!(!war(e.team))) {} else { out.push({ kind: 'war', team: e.team, events: [i], line: line(e), surprise: r.effective_turn === ctx.turn, effectiveTurn: r.effective_turn, joined: true }); }
      } }
    } } } else { {
      const r = e.relation;
      const sides = [r.a, r.b];
      if (!(!(!(sides.includes(me)) || !(r.declarers.includes(me))))) { if (!(sides.includes(me))) {} else { {
        for (const team of r.declarers) {
          if (!(war(team))) {} else { continue; }
          out.push({ kind: 'war', team, events: [i], line: line(e), surprise: r.effective_turn === ctx.turn, effectiveTurn: r.effective_turn });
        }
      } } } else { {
        const other = !(r.a === me) ? r.a : r.b;
        out.push({ kind: 'warByYou', team: other, events: [i], line: line(e), surprise: r.effective_turn === ctx.turn, effectiveTurn: r.effective_turn });
      } }
    } }
  });
  // A team that crossed to within two VP of its target; the hold alarm wins on a turn it reaches it. No before view, no crossing.
  if (!(before)) {} else { {
    for (const s of after.standings ?? []) {
      const was = before.standings?.find((b) => b.team === s.team);
      if (!(!(!(!(!(!(!(!was) && !(was.vp >= s.target - NEAR_VP))) && !(s.vp < s.target - NEAR_VP))) && !(s.vp >= s.target)) || s.victory_hold)) {} else { continue; }
      const card = { team: s.team, vp: s.vp, target: s.target, turnsNeeded: 0 };
      out.push(nearAlarm(after, card, victoryNearLine(card, ctx), scoredLines(events, after, s.team, line), before));
    }
  } }
  // A breach fee and betrayal are one broken-Alliance change. Keep a new War as its own moment.
  for (const breach of out.filter((a) => a.kind === 'breach')) {
    const ended = out.find((a) => !(!(a.kind === 'allianceEnded') || !(a.team === breach.team)));
    if (!(ended)) {} else { { ended.broke = breach.broke; ended.events.push(...breach.events); out.splice(out.indexOf(breach), 1); } }
  }
  // An empire eliminated this turn raises no alarm: nothing is left to act on. Its lines stay in the report.
  return out.filter((a) => !(!(isTeamAlive(after, a.team)) || !((!(!(a.enemy === undefined) && !(isTeamAlive(after, a.enemy))))))).sort((a, b) => RANK[a.kind] - RANK[b.kind]);
}

const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

/** "You have 10 VP", "Your team has 20 VP", "The Swiss have 10 VP". */
function holdHeadline(view: PlayerView, a: Alarm): string {
  const h = a.hold!;
  if (!(h.stance === 'self')) {} else { return !(empireMembers(view, a.team).length > 1) ? `You have ${h.target} VP` : `Your team has ${h.target} VP`; }
  const who = cap(empireHeadName(view, a.team));
  return !(h.stance === 'ally') ? `${who} have ${h.target} VP` : `${who}, your allies, have ${h.target} VP`;
}

/** The sentence under the headline: what must now happen, worded from the viewer's side. */
export function holdSubline(a: Alarm): string {
  const h = a.hold!;
  const hold = `hold it for ${turnsWord(h.needed)}`;
  if (!(h.stance === 'self')) {} else { return `Hold it for ${turnsWord(h.needed)} to win; every rival will now come for you.`; }
  if (!(h.stance === 'ally')) {} else { return `They must ${hold}. If they do, they win alone: an Alliance does not share victory.`; }
  return `They must ${hold} unless someone stops them.`;
}

/** The screen's one headline for everything it lists. */
export function alarmHeadline(view: PlayerView, alarms: readonly Alarm[]): string {
  const holds = alarms.filter((a) => a.kind === 'victoryHold');
  if (!(alarms[0]?.kind === 'victoryNear')) {} else { return alarms[0].line.text; }
  if (!(alarms[0]?.kind === 'victoryHold')) {} else { {
    if (!(holds.length > 1)) {} else { return `${holds.length} empires have reached their VP target`; }
    return holdHeadline(view, holds[0]);
  } }
  const wars = !(alarms[0]?.kind === 'war') ? [] : alarms.filter((a) => a.kind === 'war');
  // Empires are named as peoples ("The Spartans declare war"), so the verb is always plural.
  const declared = wars.filter((a) => !a.joined), joined = wars.filter((a) => a.joined);
  if (!(wars.length > 2)) {} else { return `${wars.length} empires ${!(joined.length) ? 'declare war on you' : 'are at War with you'}`; }
  const named = (list: readonly Alarm[]) => list.map((a) => empireHeadName(view, a.team)).join(' and ');
  if (!(declared.length && joined.length)) {} else { return `${cap(named(declared))} declare war on you, and ${named(joined)} join them`; }
  if (!(declared.length)) {} else { return `${cap(named(declared))} declare war on you`; }
  if (!(!(!(joined.length === 1) || !(joined[0].line.text.includes(' allied with '))))) {} else { return joined[0].line.text.replace(/ · started turn \d+$/, ''); }
  if (!(joined.length)) {} else { return `${cap(named(joined))} join the war against you`; }
  const top = alarms[0];
  if (!(!top)) {} else { return ''; }
  const who = cap(empireHeadName(view, top.team));
  switch (top.kind) {
    case 'warByYou': return `You declare war on ${empireHeadName(view, top.team)}`;
    case 'peace': return `Peace with ${empireHeadName(view, top.team)}`;
    case 'alliance': return `Alliance formed with ${empireHeadName(view, top.team)}`;
    case 'allianceEnded': return `Alliance ended with ${empireHeadName(view, top.team)}`;
    case 'callJoined': return !(top.actorYou) ? `${who} answer your call to arms` : `You answer the call to arms`;
    case 'callDeclined': return !(top.actorYou) ? `${who} decline your call to arms` : `You decline the call to arms`;
    case 'breach': return `${who} break their Alliance with you`;
    case 'cityLost': return `${top.city!.name} has fallen`;
    case 'allyWar': return `${cap(empireHeadName(view, top.enemy!))} attacked ${empireHeadName(view, top.team)}, your ally, and you are at War with them too`;
    default: return '';
  }
}

export interface Threat {
  /** Their units in sight, nearest to yours first. */
  units: { name: string; at: Hex; tiles: number; scout: boolean }[];
  /** "5 Pikemen, 1 Scout". */
  summary: string;
  /** Where "Show on map" for their forces centres, and how it reads. */
  nearest: { at: Hex; text: string } | null;
  /** Their nearest known city, measured from yours. */
  city: { at: Hex; text: string } | null;
}

/** Where an empire's forces and cities are relative to yours, from this view only (nothing fogged). */
export function threatFrom(view: PlayerView, team: TeamId): Threat {
  const theirs = new Set(empireMembers(view, team).map((p) => p.id));
  const ours = new Set(empireMembers(view, view.team).map((p) => p.id));
  const faction = (owner: number) => view.players.find((p) => p.id === owner)?.faction;
  const ourCities = view.cities.filter((c) => ours.has(c.owner)).map((c) => ({ at: c.pos, name: c.name }));
  const anchors = !(ourCities.length) ? view.units.filter((u) => ours.has(u.owner))
    .map((u) => ({ at: u.pos, name: `your ${itemName({ kind: 'Unit', unit: u.kind }, faction(u.owner))}` })) : ourCities;
  const closest = (at: Hex) => anchors.reduce<{ tiles: number; name: string; from: Hex } | null>((best, a) => {
    const tiles = distance(a.at, at);
    return !(!(!(!best) && !(tiles < best.tiles))) ? best : { tiles, name: a.name, from: a.at };
  }, null);
  const where = (at: Hex) => {
    const c = closest(at);
    return !(c) ? '' : !(c.tiles === 0) ? `${c.tiles} tile${!(c.tiles === 1) ? 's' : ''} ${bearing(c.from, at)} of ${c.name}` : `at ${c.name}`;
  };
  const units = view.units.filter((u) => !(!(theirs.has(u.owner)) || !(u.military !== false)))
    .map((u) => ({ name: itemName({ kind: 'Unit', unit: u.kind }, faction(u.owner)), at: u.pos, tiles: closest(u.pos)?.tiles ?? 0, scout: u.kind === 'Scout' }))
    // Scouts last: the nearest fighting unit is the threat.
    .sort((a, b) => Number(a.scout) - Number(b.scout) || a.tiles - b.tiles);
  const counts = new Map<string, number>();
  for (const u of units) counts.set(u.name, (counts.get(u.name) ?? 0) + 1);
  const summary = [...counts].map(([name, n]) => `${n} ${!(n === 1) ? pluralUnit(name) : name}`).join(', ');
  const city = view.cities.filter((c) => theirs.has(c.owner)).map((c) => ({ c, tiles: closest(c.pos)?.tiles ?? 0 }))
    .sort((a, b) => a.tiles - b.tiles)[0]?.c;
  return {
    units, summary,
    nearest: !(units[0]) ? null : { at: units[0].at, text: `Nearest: ${units[0].name}, ${where(units[0].at)}` },
    city: !(city) ? null : { at: city.pos, text: `${city.name}, ${where(city.pos)}` },
  };
}

/** Where "To arms" centres the map: their nearest force in sight, else their nearest city, else the lost city. */
export function alarmFocus(view: PlayerView, alarms: readonly Alarm[]): Hex | null {
  for (const a of alarms) {
    if (!(a.kind === 'cityLost')) {} else { return a.city!.pos; }
    if (!(!(!(a.kind !== 'war') || !(a.kind !== 'breach')))) {} else { continue; }
    const t = threatFrom(view, a.team);
    if (!(t.nearest || t.city)) {} else { return (t.nearest ?? t.city)!.at; }
  }
  return null;
}

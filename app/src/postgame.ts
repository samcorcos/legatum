// The post-game report's model: the core's recorded history (`PlayerView.history`, present only once a game is
// finished) arranged for the charts, the timeline and the statistics table. Presentation only: every number is
// one the core recorded; this file only picks, sums, ranks and words them.
import { ARMY_RULE, ERA_NAMES, FACTION_GOAL, itemName } from './data';
import { empireNameOf } from './empires';
import { playerInk } from './theme';
import { WONDER_INFO } from './wonders';
import type { Chronicle, Faction, GameHistory, PlayerId, PlayerView, TeamId } from './types';

export type Metric = 'vp' | 'cities' | 'population' | 'science' | 'gold' | 'army';

export const METRICS: Record<Metric, { label: string; short: string; icon: string; note: string }> = {
  vp: { label: 'Victory points', short: 'VP', icon: 'vp', note: 'VP at the end of each turn' },
  cities: { label: 'Cities', short: 'Cities', icon: 'city', note: 'Cities held at the end of each turn' },
  population: { label: 'Population', short: 'Population', icon: 'emblem-growth', note: 'Population of all cities' },
  science: { label: 'Science', short: 'Science', icon: 'yield-science', note: 'Science produced each turn' },
  gold: { label: 'Gold', short: 'Gold', icon: 'yield-gold', note: 'Gold in the treasury at the end of each turn' },
  army: { label: 'Military', short: 'Military', icon: 'title-strongest-army', note: ARMY_RULE },
};
/** The "Empires over time" chart's choices, in the order its switcher lists them. */
export const GROWTH_METRICS: Metric[] = ['cities', 'population', 'science', 'gold', 'army'];

export interface ReportEmpire {
  id: PlayerId;
  team: TeamId;
  faction: Faction;
  /** The empire's name as every surface writes it ("Switzerland", or "Switzerland · AI 5" for a twin). */
  name: string;
  /** "You" for the viewer, else the name. */
  label: string;
  /** Parchment-safe seat ink for lines and marks. */
  ink: string;
  mine: boolean;
  winner: boolean;
  alive: boolean;
  /** The turn the empire fell, if it did. */
  fell?: number;
}

export interface EraBand {
  /** 1-based era (2 Classical .. 8 Future). */
  era: number;
  name: string;
  /** The turn the first empire entered it. */
  turn: number;
  by: PlayerId | null;
}

export type LaneKind = 'war' | 'capture';
export interface Marker {
  turn: number;
  lane: LaneKind;
  icon: string;
  text: string;
  /** Whose mark it is (its colour). */
  who: PlayerId | null;
  mine: boolean;
}

type Tone = 'mine' | 'against' | 'neutral';
export interface TimelineEntry {
  turn: number;
  kind: Chronicle['kind'];
  icon: string;
  text: string;
  /** Empires the line is about (their portraits lead the line). */
  who: PlayerId[];
  tone: Tone;
  /** Involves the viewer (or the viewer's team). */
  yours: boolean;
  /** A turning point of the whole game (shown in "Your story" even when it is not yours). */
  major: boolean;
}

const MAJOR = new Set<Chronicle['kind']>(['Won', 'Drawn', 'Eliminated', 'HoldStarted', 'HoldBroken', 'GreatWork', 'Goal']);

export interface StatRow {
  empire: ReportEmpire;
  vp: number;
  peakVp: number;
  cities: number;
  peakCities: number;
  population: number;
  /** Science produced over the whole game. */
  science: number;
  techs: number;
  peakArmy: number;
  founded: number;
  captured: number;
  lost: number;
  wars: number;
}

export interface PostGame {
  firstTurn: number;
  lastTurn: number;
  /** Ordered: winners first, then by final VP, then seat. */
  empires: ReportEmpire[];
  /** The VP target, when every team shares one (the chart draws it). */
  target: number | null;
  eras: EraBand[];
  markers: Marker[];
  timeline: TimelineEntry[];
  stats: StatRow[];
  /** Whether the record starts mid-game (a save from before history was kept). */
  partial: boolean;
}

const pluralName = (name: string) => /^(Mongols|Celts)\b/.test(name);

/** One series of one empire, cut short after it fell (a fallen empire's line ends where it died). */
export function seriesOf(history: GameHistory, metric: Metric, id: PlayerId): number[] {
  return history.empires[id]?.[metric] ?? [];
}

/** The last index of a series to draw: an eliminated empire's line stops at the turn it fell. */
export function drawnLength(pg: PostGame, history: GameHistory, id: PlayerId): number {
  const n = seriesOf(history, 'vp', id).length;
  const e = pg.empires.find((x) => x.id === id);
  return e?.fell !== undefined ? Math.max(0, Math.min(n, e.fell - history.first_turn + 1)) : n;
}

export function postGame(view: PlayerView): PostGame | null {
  const h = view.history;
  if (!h || !h.empires.length || !h.empires[0]?.vp.length) return null;
  const n = h.empires[0].vp.length;
  const lastTurn = h.first_turn + n - 1;
  const chronicle = h.chronicle ?? [];
  const myTeam = view.team;
  const fell = new Map<PlayerId, number>();
  for (const c of chronicle) if (c.kind === 'Eliminated' && !fell.has(c.player)) fell.set(c.player, c.turn);
  const last = (id: PlayerId, m: Metric) => seriesOf(h, m, id).at(-1) ?? 0;
  const empires: ReportEmpire[] = view.players.map((p) => {
    const name = empireNameOf([p], view.players);
    return {
      id: p.id, team: p.team, faction: p.faction, name,
      label: p.id === view.me ? 'You' : name,
      ink: playerInk(p.id), mine: p.team === myTeam, winner: view.winner !== null && p.team === view.winner,
      alive: p.alive, ...(fell.has(p.id) || !p.alive ? { fell: fell.get(p.id) ?? lastTurn } : {}),
    };
  });
  empires.sort((a, b) => Number(b.winner) - Number(a.winner) || last(b.id, 'vp') - last(a.id, 'vp') || Number(b.alive) - Number(a.alive) || a.id - b.id);
  const byId = new Map(empires.map((e) => [e.id, e]));
  const nameOf = (id: PlayerId | null | undefined) => id === null || id === undefined ? 'Nobody' : id >= 255 ? 'Barbarians' : byId.get(id)?.label ?? `Empire ${id + 1}`;
  const teamName = (team: TeamId) => {
    const members = view.players.filter((p) => p.team === team);
    if (team === myTeam) return members.length > 1 ? 'Your team' : 'You';
    return empireNameOf(members, view.players) || `Team ${team}`;
  };
  const targets = new Set(view.standings.map((s) => s.target));
  const eras = eraBands(h, chronicle);
  const timeline = chronicle.flatMap((c) => {
    const t = timelineEntry(c, view, nameOf, teamName);
    return t ? [{ ...t, major: MAJOR.has(c.kind) || (c.kind === 'Era' && c.first) }] : [];
  });
  const markers = chronicle.flatMap((c) => marker(c, view, nameOf, teamName));
  return {
    firstTurn: h.first_turn, lastTurn, empires,
    target: targets.size === 1 ? [...targets][0] : null,
    eras, markers, timeline,
    stats: empires.map((e) => statRow(e, h, chronicle)),
    partial: h.first_turn > 1,
  };
}

/** The turn the first empire entered each era: from the era series, else the chronicle. */
export function eraBands(h: GameHistory, chronicle: Chronicle[]): EraBand[] {
  const bands = new Map<number, EraBand>();
  h.empires.forEach((e, id) => e.era?.forEach((era, i) => {
    const turn = h.first_turn + i;
    for (let x = 2; x <= era; x++) if (!bands.has(x) || bands.get(x)!.turn > turn) bands.set(x, { era: x, name: ERA_NAMES[x - 1] ?? `Era ${x}`, turn, by: id });
  }));
  for (const c of chronicle) {
    if (c.kind !== 'Era' || bands.has(c.era)) continue;
    bands.set(c.era, { era: c.era, name: ERA_NAMES[c.era - 1] ?? `Era ${c.era}`, turn: c.turn, by: c.player });
  }
  return [...bands.values()].sort((a, b) => a.era - b.era);
}

function timelineEntry(c: Chronicle, view: PlayerView, nameOf: (id: PlayerId | null | undefined) => string, teamName: (t: TeamId) => string): Omit<TimelineEntry, 'major'> | null {
  const team = (id: PlayerId | null | undefined) => (id === null || id === undefined ? null : view.players.find((p) => p.id === id)?.team ?? null);
  const yoursP = (...ids: (PlayerId | null | undefined)[]) => ids.some((id) => team(id) === view.team);
  const base = { turn: c.turn, kind: c.kind };
  switch (c.kind) {
    case 'Founded':
      return { ...base, icon: 'emblem-founded', text: `${nameOf(c.player)} founded ${c.city}`, who: [c.player], tone: yoursP(c.player) ? 'mine' : 'neutral', yours: yoursP(c.player) };
    case 'Captured': {
      const lost = yoursP(c.from) && !yoursP(c.by);
      const text = lost ? `${nameOf(c.by)} captured ${c.city} from ${c.from === view.me ? 'you' : nameOf(c.from)}` : `${nameOf(c.by)} captured ${c.city} from ${nameOf(c.from)}`;
      return { ...base, icon: 'emblem-captured', text, who: [c.by, c.from].filter((x) => x < 255), tone: lost ? 'against' : yoursP(c.by) ? 'mine' : 'neutral', yours: yoursP(c.by, c.from) };
    }
    case 'Razed': {
      const lost = yoursP(c.from) && !yoursP(c.by);
      return { ...base, icon: 'ui-alert', text: c.ordered ? `${nameOf(c.by)} razed ${c.city}` : `${nameOf(c.by)} sacked ${c.city}${lost ? ', one of your cities' : `, a city of ${nameOf(c.from)}`}`, who: [c.by, c.from].filter((x) => x < 255), tone: lost ? 'against' : yoursP(c.by) ? 'mine' : 'neutral', yours: yoursP(c.by, c.from) };
    }
    case 'War': {
      const declarer = c.by ?? null;
      const other = declarer === c.a ? c.b : declarer === c.b ? c.a : null;
      const onYou = other === view.team;
      const text = declarer !== null && other !== null
        ? `${teamName(declarer)} declared war on ${onYou ? (teamName(other) === 'You' ? 'you' : 'your team') : teamName(other)}`
        : `War between ${teamName(c.a)} and ${teamName(c.b) === 'You' ? 'you' : teamName(c.b)}`;
      const yours = c.a === view.team || c.b === view.team;
      return { ...base, icon: 'unit-spearman', text: text.replace(/^War between You and/, 'War between you and'), who: leaders(view, [c.a, c.b]), tone: yours ? (declarer === view.team ? 'mine' : 'against') : 'neutral', yours };
    }
    case 'Peace': {
      const yours = c.a === view.team || c.b === view.team;
      const [x, y] = c.b === view.team ? [c.b, c.a] : [c.a, c.b];
      const text = x === view.team ? `${teamName(x)} made peace with ${teamName(y)}` : `Peace between ${teamName(x)} and ${teamName(y)}`;
      return { ...base, icon: 'treaty-peace', text, who: leaders(view, [c.a, c.b]), tone: yours ? 'mine' : 'neutral', yours };
    }
    case 'Era': {
      const name = ERA_NAMES[c.era - 1] ?? `era ${c.era}`;
      return { ...base, icon: 'emblem-tech', text: c.first ? `${nameOf(c.player)} entered the ${name} era first` : `${nameOf(c.player)} entered the ${name} era`, who: [c.player], tone: yoursP(c.player) ? 'mine' : 'neutral', yours: yoursP(c.player) };
    }
    case 'GreatWork':
      return { ...base, icon: 'emblem-greatwork', text: `${nameOf(c.player)} completed the ${itemName({ kind: 'Building', building: c.building })} in ${c.city} (+2 VP)`, who: [c.player], tone: yoursP(c.player) ? 'mine' : 'neutral', yours: yoursP(c.player) };
    case 'Wonder':
      return { ...base, icon: 'emblem-ruin', text: `${nameOf(c.player)} discovered ${WONDER_INFO[c.wonder]?.name ?? c.wonder}`, who: [c.player], tone: yoursP(c.player) ? 'mine' : 'neutral', yours: yoursP(c.player) };
    case 'Objective': {
      const name = ERA_NAMES[c.era - 1] ?? `era ${c.era}`;
      return { ...base, icon: 'emblem-milestone', text: `${nameOf(c.player)} scored the ${name} Age: ${c.objective} (+1 VP)`, who: [c.player], tone: yoursP(c.player) ? 'mine' : 'neutral', yours: yoursP(c.player) };
    }
    case 'Goal': {
      const faction = view.players.find((p) => p.id === c.player)?.faction;
      const goal = faction ? FACTION_GOAL[faction].name : 'its faction goal';
      return { ...base, icon: 'emblem-milestone', text: `${nameOf(c.player)} completed ${goal}`, who: [c.player], tone: yoursP(c.player) ? 'mine' : 'neutral', yours: yoursP(c.player) };
    }
    case 'Eliminated': {
      const name = nameOf(c.player);
      return { ...base, icon: 'ui-cross', text: name === 'You' ? 'Your empire fell' : `${name} ${pluralName(name) ? 'were' : 'was'} eliminated`, who: [c.player], tone: yoursP(c.player) ? 'against' : 'neutral', yours: yoursP(c.player) };
    }
    case 'HoldStarted': {
      const name = teamName(c.team);
      return { ...base, icon: 'vp', text: `${name} reached the VP target and began to hold it`, who: leaders(view, [c.team]), tone: c.team === view.team ? 'mine' : 'against', yours: c.team === view.team };
    }
    case 'HoldBroken': {
      const name = teamName(c.team);
      return { ...base, icon: 'vp', text: `${name} dropped below the VP target: the hold was broken`, who: leaders(view, [c.team]), tone: c.team === view.team ? 'against' : 'neutral', yours: c.team === view.team };
    }
    case 'Won':
      return { ...base, icon: 'emblem-milestone', text: `${teamName(c.team)} won. ${c.reason.replace(/\.?$/, '.')}`, who: leaders(view, [c.team]), tone: c.team === view.team ? 'mine' : 'against', yours: true };
    case 'Drawn':
      return { ...base, icon: 'emblem-milestone', text: 'The game ended in a draw', who: [], tone: 'neutral', yours: true };
    default:
      return null;
  }
}

const leaders = (view: PlayerView, teams: TeamId[]): PlayerId[] => teams.flatMap((t) => view.players.filter((p) => p.team === t).slice(0, 1).map((p) => p.id));

/** The VP chart's event lanes: wars declared, cities captured or sacked. */
function marker(c: Chronicle, view: PlayerView, nameOf: (id: PlayerId | null | undefined) => string, teamName: (t: TeamId) => string): Marker[] {
  const team = (id: PlayerId | null | undefined) => (id === null || id === undefined ? null : view.players.find((p) => p.id === id)?.team ?? null);
  if (c.kind === 'War') {
    const by = c.by ?? c.a;
    const other = by === c.a ? c.b : c.a;
    const who = view.players.find((p) => p.team === by)?.id ?? null;
    const text = c.by !== null && c.by !== undefined ? `Turn ${c.turn}: ${teamName(by)} declared war on ${teamName(other)}` : `Turn ${c.turn}: war between ${teamName(c.a)} and ${teamName(c.b)}`;
    return [{ turn: c.turn, lane: 'war', icon: 'unit-spearman', text, who, mine: c.a === view.team || c.b === view.team }];
  }
  if (c.kind === 'Captured' || c.kind === 'Razed') {
    const text = `Turn ${c.turn}: ${nameOf(c.by)} ${c.kind === 'Captured' ? 'captured' : c.ordered ? 'razed' : 'sacked'} ${c.city} (${nameOf(c.from)})`;
    return [{ turn: c.turn, lane: 'capture', icon: c.kind === 'Captured' ? 'emblem-captured' : 'ui-alert', text, who: c.by < 255 ? c.by : null, mine: team(c.by) === view.team || team(c.from) === view.team }];
  }
  return [];
}

function statRow(e: ReportEmpire, h: GameHistory, chronicle: Chronicle[]): StatRow {
  const s = h.empires[e.id];
  const max = (xs: number[] | undefined) => (xs?.length ? Math.max(...xs) : 0);
  const team = e.team;
  let founded = 0, captured = 0, lost = 0, wars = 0;
  for (const c of chronicle) {
    if (c.kind === 'Founded' && c.player === e.id) founded++;
    if ((c.kind === 'Captured' || (c.kind === 'Razed' && !c.ordered)) && c.by === e.id) captured++;
    if ((c.kind === 'Captured' || (c.kind === 'Razed' && !c.ordered)) && c.from === e.id) lost++;
    if (c.kind === 'Razed' && c.ordered && c.by === e.id) lost++;
    if (c.kind === 'War' && (c.a === team || c.b === team)) wars++;
  }
  return {
    empire: e,
    vp: s?.vp.at(-1) ?? 0, peakVp: max(s?.vp),
    cities: s?.cities.at(-1) ?? 0, peakCities: max(s?.cities),
    population: s?.population.at(-1) ?? 0,
    science: s?.science.reduce((a, b) => a + b, 0) ?? 0,
    techs: s?.techs.at(-1) ?? 0,
    peakArmy: max(s?.army),
    founded, captured, lost, wars,
  };
}

/** Era of a turn: the last band begun at or before it (1 = the first era). */
export function eraAt(bands: EraBand[], turn: number): number {
  let era = 1;
  for (const b of bands) if (b.turn <= turn) era = b.era;
  return era;
}

/** The chronicle grouped by era, newest era last, for the timeline's headings. */
export function timelineByEra(pg: PostGame, entries: TimelineEntry[]): { era: number; name: string; from: number; to: number; entries: TimelineEntry[] }[] {
  const groups = new Map<number, TimelineEntry[]>();
  for (const t of entries) {
    const era = eraAt(pg.eras, t.turn);
    if (!groups.has(era)) groups.set(era, []);
    groups.get(era)!.push(t);
  }
  const start = (era: number) => (era === 1 ? pg.firstTurn : pg.eras.find((b) => b.era === era)?.turn ?? pg.firstTurn);
  const eras = [...groups.keys()].sort((a, b) => a - b);
  return eras.map((era) => {
    const next = pg.eras.find((b) => b.era > era);
    return { era, name: ERA_NAMES[era - 1] ?? `Era ${era}`, from: start(era), to: next ? next.turn - 1 : pg.lastTurn, entries: groups.get(era)! };
  });
}

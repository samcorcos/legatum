// The victory race as data: every way to score, its rule, who holds it, and the public numbers
// behind it, from the viewer's PlayerView alone (nothing fogged leaks). The race panel's tokens, the
// token cards and the objective-cards sheet all read this, so a source is worded and owned the
// same way everywhere. Rules, names and icons come from data.ts; nothing here is a rule.
//
// No suggestions: this model has no routes, no "fastest", no ranked next points. It says what is
// held, by whom, and what each empire's public number is.
import type { Faction, Hex, ObjectiveFamily, PlayerId, PlayerView, UnitView } from './types';
import {
  BUILDING_NAME, FACTION_GOAL, FACTION_INFO, GREAT_WORK_ICON, GREAT_WORK_NAME, GREAT_WORK_RULE,
  factionGoalRule, factionGoalTerms, factionGoalVp, decadenceRules, factionObjectiveText, isGreatWork, type DecadenceRules,
} from './data';
import { roman } from './brand/ornaments';
import { isBarbarian } from './barbarians';
import { cappedGoalLabel } from './ui/goalText';
import { empireOfPlayer, livingPlayers } from './empires';
import { playerColor, playerInk } from './theme';
import { buildTracks, teamHold, type TeamHold } from './vp';
import { sakokuLine, sakokuStatus, type SakokuStatus } from './sakoku';
import { aiLevel } from './aiLevel';

type SourceKind = 'work' | 'goal' | 'era' | 'capital';
/** From the viewer's side: mine (or my team's), a rival's, or nobody's yet. */
type Own = 'mine' | 'theirs' | 'open';

export interface Fact {
  id: PlayerId;
  /** A number to draw a bar from; null when it is not public. */
  value: number | null;
  label: string;
}

export interface Progress { value: number; target: number; label: string }

/** One of an era's three public objectives: who has scored the era with it, and the viewer's own progress. */
export interface EraObjective {
  slot: number;
  family: ObjectiveFamily;
  text: string;
  target: number;
  kind?: string;
  /** The rule behind the text, as the engine serializes it. */
  objective?: { kind: string; [field: string]: unknown };
  /** World Ages: the price printed on the card in its own currency. */
  /** Empires that scored the era with this objective (public). */
  scorers: PlayerId[];
  /** The viewer's own progress (only for eras entered, plus the next one). */
  mine?: { current: number; target: number; ready?: boolean | null;
    countBaseline?: number; countTotal?: number; countingSinceTurn?: number; restartEras?: number[] };
}

/** An era as a way to score: its three objectives, who has entered it, and the viewer's standing in it. */
interface EraInfo {
  era: number;
  /** The era's name from the core ("Consolidation"). */
  eraName: string;
  entered: PlayerId[];
  objectives: EraObjective[];
  /** The viewer has entered the era (can score it). */
  myEntered: boolean;
  /** Living empires that scored it, whichever card they used (a save may name a card no longer drawn). */
  scored: PlayerId[];
  /** The viewer has scored it: `scored` includes the viewer. */
  myScored: boolean;
}

export interface Source {
  id: string;
  kind: SourceKind;
  name: string;
  icon: string;
  vp: number;
  /** The rule, plainly: what it takes. */
  rule: string;
  /** Short terms: "Kept for good · once each". */
  terms: string;
  /** A live source can be lost (a held world wonder; a live faction goal): its token is outlined, not solid. */
  live: boolean;
  owners: PlayerId[];
  own: Own;
  /** Faction goals: the one empire that can earn it. */
  only?: PlayerId;
  faction?: Faction;
  /** Public facts behind it, per empire. */
  facts: Fact[];
  unit: string;
  /** Where the facts stop being public. */
  privacy?: string;
  /** Your own progress where only you can see it. */
  progress?: Progress;
  /** A held goal (Byzantium's Heir, Mongols's Courier Hall): whoever holds the item scores it, so `owners` are the holders, not the faction. */
  held?: boolean;
  /** Held goals: who holds the item now, and where it stands when the viewer can place it (public facts only). */
  holdNote?: string;
  heldAt?: Hex;
  /** Faction goals: every condition the core checks, where the card's rule leaves out fine print. */
  detail?: string;
  /** Eras: the objectives and who has scored them. */
  era?: EraInfo;
}

export interface VpEmpire {
  id: PlayerId;
  team: number;
  name: string;
  faction: Faction;
  /** The actual AI level from the game setup; null for human seats. */
  aiLevel: string | null;
  /** The player colour, for fills and tints. */
  color: string;
  /** A shade of it that holds on parchment, for outlines, bands and bars. */
  ink: string;
  /** This seat. */
  isMe: boolean;
  /** This seat or its team-mate. */
  isMine: boolean;
  /** Sakoku: this empire has closed the country (public), with its countdown. */
  sakoku?: SakokuStatus | null;
  /** Its current Gold, public to every player. */
  gold: number;
  /** Its original capital still stands in its team's hands (public); the last team that holds one wins. */
  capitalStands: boolean;
  /** Public Power and its lead over the average empire, used when decadence is active. */
  power: number;
  lead: number;
  /** The percent cut from yields when decadence is active. */
  decadence: number;
  /** The sources it holds, in token order (kept ones first, then live ones). */
  held: Source[];
  goal: Source;
}

/** `of`: for a faction goal, the empire it belongs to (a held goal sits in its holder's row but wears this crest). */
export interface Token { source: Source; owner: VpEmpire; of?: VpEmpire }

export interface VpRow {
  team: number;
  label: string;
  members: VpEmpire[];
  vp: number;
  target: number;
  rank: number;
  isMine: boolean;
  /** Everything the row's empires hold, member by member. */
  tokens: Token[];
  /** Goals not done yet: their crests close the row, dashed. */
  pendingGoals: Token[];
  /** Some member's original capital still stands in the team's hands: the team can still win by holding the last capital. */
  capitalStands: boolean;
  /** The countdown to this team winning, while it holds its target. */
  hold: TeamHold | null;
}

export interface VpModel {
  /** Public held points: the rival original capitals each empire holds now. */
  capitals: { owner: PlayerId; names: string[] }[];
  gold?: number;
  me: PlayerId;
  turn: number;
  rows: VpRow[];
  empires: VpEmpire[];
  works: Source[];
  capitalSources: Source[];
  /** Every era after the first (VP v2), in order. */
  eras: Source[];
  goals: Source[];
  /** The viewer's era and optional decadence setting. */
  raceRules: { era: number; decadence: number; rules: DecadenceRules };
  /** Every way to score, in the order the arrow keys flip through it: works, capitals, eras, goals in race order. */
  all: Source[];
  /** Works and eras nobody holds. */
  open: Source[];
  /** Teams holding their VP target, soonest win first: the countdown every surface shows. */
  holds: VpRow[];
  byId: Map<string, Source>;
}

const goalId = (id: PlayerId) => `goal-${id}`;
const eraId = (era: number) => `era-${era}`;
/** An era's medallion: the advancement symbol. */
const ERA_ICON = 'milestone-six-techs';

/** An era shows in the race once some empire has entered or scored it; a future era nobody has reached is only on the objectives sheet. */
const eraInRace = (s: Source) => s.owners.length > 0 || (s.era?.entered.length ?? 0) > 0;

/** Who holds a source, in words: "Yours", "Nobody yet", "Not yet", a name, "2 empires". */
export function ownWords(model: Pick<VpModel, 'empires'>, s: Source): string {
  const short = (name: string) => (name.length > 11 ? name.split(' ')[0] : name);
  if (s.own === 'open') return s.kind === 'goal' ? (s.held ? 'Nobody yet' : 'Not yet') : 'Nobody yet';
  if (s.kind === 'goal' && s.held) return s.own === 'mine' ? 'Yours · held' : short(model.empires.find((x) => x.id === s.owners[0])!.name);
  if (s.kind === 'goal') return s.own === 'mine' ? 'Yours · done' : 'Done';
  if (s.kind === 'era' && s.own === 'mine' && s.owners.length === 1 && s.owners[0] === model.empires.find((e) => e.isMe)?.id) return 'Scored';
  // A shared work names its holders: "You and Mongols", "Yours and 2 more", "Mongols, Byzantium", "5 empires".
  const names = s.owners.map((id) => model.empires.find((x) => x.id === id)!).sort((a, b) => Number(b.isMine) - Number(a.isMine));
  if (s.own === 'mine') {
    if (names.length === 1) return 'Yours';
    return names.length === 2 ? `You and ${short(names[1].name)}` : `Yours and ${names.length - 1} more`;
  }
  return names.length === 1 ? short(names[0].name) : names.length === 2 ? `${short(names[0].name)}, ${short(names[1].name)}` : `${names.length} empires`;
}

export function buildVpModel(view: PlayerView): VpModel {
  const me = view.me;
  const alive = livingPlayers(view);
  const isAlive = (id: PlayerId) => alive.some((p) => p.id === id);
  const team = new Set(view.players.filter((p) => p.team === view.team).map((p) => p.id));
  const own = (owners: PlayerId[]): Own => (owners.some((o) => team.has(o)) ? 'mine' : owners.length ? 'theirs' : 'open');

  const mePublic = view.players.find((p) => p.id === me);
  const workOwners = alive.map((p) => ({ player: p, works: p.vp_sources.flatMap(([src]) => src.kind === 'GreatWork' ? [src.of] : []).sort((a, b) => a - b) }));
  const owners = alive.filter((p) => p.vp_sources.some(([src]) => src.kind === 'GreatWork')).map((p) => p.id);
  // A world wonder under construction is public to the builder's team and to empires that have met it.
  const workFacts: Fact[] = alive.map((p) => {
    const w = view.great_works?.find((g) => g.player === p.id);
    const primaryOf = workOwners.find((entry) => entry.player.id === p.id)?.works[0];
    const heldSite = view.tiles.find((t) => t.landmark && t.landmark.of === primaryOf && t.owner === p.id);
    if (heldSite?.landmark) return { id: p.id, value: 1, label: BUILDING_NAME[heldSite.landmark.building] };
    const heldWork = view.great_works?.find((g) => g.player === primaryOf);
    if (heldWork?.completed) return { id: p.id, value: 1, label: BUILDING_NAME[heldWork.completed] };
    if (w?.completed && owners.includes(p.id)) return { id: p.id, value: 1, label: BUILDING_NAME[w.completed] };
    if (w?.building && (team.has(p.id) || view.diplomacy.met?.includes(p.team))) return { id: p.id, value: 1, label: BUILDING_NAME[w.building] };
    return { id: p.id, value: 0, label: '' };
  });
  const baseWork: Source = {
    id: 'GreatWork', kind: 'work', name: GREAT_WORK_NAME, icon: GREAT_WORK_ICON, vp: 2,
    rule: GREAT_WORK_RULE, terms: 'Held · whoever holds the city or tile scores · capturable',
    live: true, held: true, owners, own: own(owners), facts: workFacts, unit: 'Production',
  };
  // One empire can capture several world wonders. Give each point pair its own held mark;
  // the first uses the familiar world wonder row, and further works get named rows.
  const extraWorks: Source[] = workOwners.flatMap(({ player, works }) => works.slice(1).map((of): Source => {
    const building = view.great_works?.find((w) => w.player === of)?.completed
      ?? view.tiles.find((t) => t.landmark?.of === of && isGreatWork(t.landmark.building))?.landmark?.building;
    const owners = [player.id];
    return { id: `GreatWork-${of}`, kind: 'work', name: building ? BUILDING_NAME[building] : 'Captured world wonder',
      icon: GREAT_WORK_ICON, vp: 2, rule: GREAT_WORK_RULE,
      terms: 'Held · follows its wonder tile', live: true, held: true, owners, own: own(owners),
      facts: [{ id: player.id, value: 1, label: building ? BUILDING_NAME[building] : 'World wonder' }], unit: 'Production' };
  }));
  const works = [baseWork, ...extraWorks];
  const capitalSources: Source[] = alive.flatMap((p) => (p.capitals_taken ?? []).map(([id, name]) => ({
    id: `capital-${p.id}-${id}`, kind: 'capital' as const, name, icon: 'city', vp: 1,
    rule: `${p.name} holds the original capital ${name}: 1 VP while holding it`,
    terms: 'Held · follows the city', live: true, held: true,
    owners: [p.id], own: own([p.id]), facts: [{ id: p.id, value: 1, label: name }], unit: '',
  })).concat((p.capitals_sacked ?? []).map(([id, name]) => ({
    id: `capital-sacked-${p.id}-${id}`, kind: 'capital' as const, name, icon: 'city', vp: 1,
    rule: `${p.name} sacked the original capital ${name}: 1 VP, kept for good`,
    terms: 'Kept · the city is gone', live: false, held: false,
    owners: [p.id], own: own([p.id]), facts: [{ id: p.id, value: 1, label: name }], unit: '',
  }))));

  // VP v2 eras: the public objectives drawn at game start. An era is a way to score for every empire that enters it.
  const myEra = mePublic?.era ?? 1;
  const progress = view.objective_progress ?? [];
  const eras: Source[] = (view.eras ?? []).map((ev) => {
    const scored = ev.scored_by.filter((x) => isAlive(x.player));
    const owners = [...new Set(scored.map((x) => x.player))];
    const objectives: EraObjective[] = ev.objectives.map((o) => {
      const pr = progress.find((x) => x.era === ev.era && x.slot === o.slot);
      const accounting = view.objective_accounting?.find((x) => x.era === ev.era && x.slot === o.slot);
      return { slot: o.slot, family: o.family, kind: o.objective.kind, text: factionObjectiveText(o.text, mePublic?.faction), target: o.target, scorers: scored.filter((x) => x.slot === o.slot).map((x) => x.player), objective: o.objective, mine: pr ? { current: pr.current, target: pr.target, ready: pr.ready,
        countBaseline: accounting?.baseline, countTotal: accounting?.total, countingSinceTurn: accounting?.counting_since_turn, restartEras: accounting?.restart_eras } : undefined };
    });
    const info: EraInfo = {
      era: ev.era, eraName: ev.name, entered: ev.entered_by.filter(isAlive), objectives,
      myEntered: true, scored: owners, myScored: owners.includes(me),
    };
    return {
      id: eraId(ev.era), kind: 'era', name: `Age ${roman(ev.era)}: ${ev.name}`, icon: ERA_ICON, vp: 1, rule: 'Score this Age’s card whenever you like; 1 VP, kept for good',
      terms: 'Kept for good · once per Age · the card never expires', live: false, owners, own: own(owners),
      facts: alive.map((p) => ({ id: p.id, value: owners.includes(p.id) ? 2 : ev.entered_by.includes(p.id) ? 1 : 0, label: owners.includes(p.id) ? 'Scored' : 'Open' })),
      unit: '', era: info,
    };
  });

  const nameOfId = (id: PlayerId) => (id === me ? 'You' : empireOfPlayer(view, id));
  // A held goal whose faction has fallen still scores for its holder: its card stays, in the holder's row.
  const fallen = view.players.filter((p) => !p.alive && FACTION_GOAL[p.faction].held
    && alive.some((h) => h.vp_sources.some(([src]) => src.kind === 'FactionGoal' && src.of === p.id)));
  const goals: Source[] = [...alive, ...fallen].map((p) => {
    const g = FACTION_GOAL[p.faction];
    const held = !!g.held;
    // A held goal scores for whoever holds the item: the empires whose public VP sources name this goal.
    const owners = held
      ? alive.filter((h) => h.vp_sources.some(([src]) => src.kind === 'FactionGoal' && src.of === p.id)).map((h) => h.id)
      : p.faction_goal_done ? [p.id] : [];
    const s: Source = {
      id: goalId(p.id), kind: 'goal', name: g.name, icon: g.icon, vp: factionGoalVp(p.faction),
      rule: factionGoalRule(p.faction), detail: g.detail(),
      terms: `${FACTION_INFO[p.faction].name} only · ${held ? 'Held goal' : factionGoalTerms(p.faction)}`,
      live: g.live || held, owners, own: own(owners), only: p.id, faction: p.faction, facts: [], unit: '', held: held || undefined,
    };
    if (held) {
      const where = heldPlace(view, p.id, p.faction);
      s.holdNote = heldHoldNote(view, p.faction, owners, where, nameOfId);
      if (where) s.heldAt = where.pos;
    }
    const closed = sakokuStatus(p);
    if (p.id === me) s.progress = ownGoalProgress(view);
    else if (p.goal_rule === 'Sakoku') {
      // The vow is public: the closed state, the countdown and whether the VP holds are everyone's facts.
      s.facts = [{ id: p.id, value: closed?.turns ?? 0, label: closed ? sakokuLine(closed) : 'Country open' }];
      s.privacy = 'Lost if it reopens the country';
    } else if (!held && !p.faction_goal_done) {
      if (g.publicCount) s.facts = [{ id: p.id, value: p.cities, label: `${p.cities} of ${g.publicCount.target} ${g.publicCount.unit} (public count)` }];
      s.privacy = g.live ? 'Whether it holds is private; it is lost the turn it stops holding' : 'Progress is private until it is done';
    }
    return s;
  });

  const byId = new Map<string, Source>([...works, ...capitalSources, ...eras, ...goals].map((s) => [s.id, s]));
  const nameOf = nameOfId;
  const empires: VpEmpire[] = alive.map((p) => {
    const goal = byId.get(goalId(p.id))!;
    // A held goal sits with its holder, not its faction: any goal the empire's colours score for counts as its own token.
    const kept = [...works, ...capitalSources, ...eras, ...goals].filter((s) => s.owners.includes(p.id) && !s.live);
    const live = [...works, ...capitalSources, ...goals].filter((s) => s.owners.includes(p.id) && s.live);
    return {
      id: p.id, team: p.team, name: nameOf(p.id), faction: p.faction, aiLevel: aiLevel(p), color: playerColor(p.id), ink: playerInk(p.id),
      isMe: p.id === me, isMine: team.has(p.id), power: p.power ?? 0, lead: p.lead_percent ?? 0, decadence: p.decadence_percent ?? 0, gold: p.gold ?? 0, capitalStands: p.capital_stands ?? true, sakoku: sakokuStatus(p), held: [...kept, ...live], goal,
    };
  });

  const fallenEmpire = (id: PlayerId | undefined): VpEmpire | undefined => {
    const p = fallen.find((x) => x.id === id);
    if (!p) return undefined;
    const goal = byId.get(goalId(p.id))!;
    return { id: p.id, team: p.team, name: nameOf(p.id), faction: p.faction, aiLevel: aiLevel(p), color: playerColor(p.id), ink: playerInk(p.id), isMe: false, isMine: false, power: 0, lead: 0, decadence: 0, gold: 0, capitalStands: p.capital_stands ?? true, held: [], goal };
  };

  const rows: VpRow[] = buildTracks(view, { aliveOnly: true }).map((t) => {
    const members = t.members.map((m) => empires.find((e) => e.id === m.id)).filter((e): e is VpEmpire => !!e);
    return {
      team: t.team, label: t.isMine ? (t.members.length > 1 ? 'Your team' : 'You') : t.label, members, vp: t.vp, target: t.target, rank: t.rank, isMine: t.isMine,
      tokens: members.flatMap((e) => e.held.map((source): Token => ({ source, owner: e, of: source.kind === 'goal' ? empires.find((x) => x.id === source.only) ?? fallenEmpire(source.only) : undefined }))),
      pendingGoals: members.filter((e) => e.goal.own === 'open').map((e) => ({ source: e.goal, owner: e })),
      capitalStands: members.some((e) => e.capitalStands),
      hold: (() => { const st = view.standings.find((x) => x.team === t.team); return st ? teamHold(st) : null; })(),
    };
  });

  const ordered = rows.flatMap((r) => r.members.map((e) => e.goal));
  const raceEras = eras.filter(eraInRace);
  const all = [...works, ...capitalSources, ...raceEras, ...ordered];
  const open = [...works, ...raceEras].filter((s) => s.own === 'open');
  return { me, turn: view.turn, rows, empires, works, eras, capitalSources,
    capitals: view.players.map((p) => ({ owner: p.id, names: [...(p.capitals_taken ?? []).map(([, name]) => name), ...(p.capitals_sacked ?? []).map(([, name]) => `${name} (sacked)`)] })),
    gold: view.economy.gold, raceRules: { era: myEra, decadence: mePublic?.decadence_percent ?? 0, rules: decadenceRules(view.rule_experiments) },
    goals, all, open, byId,
    holds: rows.filter((r) => r.hold).sort((a, b) => a.hold!.left - b.hold!.left || Number(b.isMine) - Number(a.isMine)) };
}

/** Only you can see your own goal's progress: the core's `faction_goal`, for every faction. Nothing here derives it. */
function ownGoalProgress(view: PlayerView): Progress | undefined {
  const g = view.faction_goal;
  return g ? { value: g.value, target: g.target, label: cappedGoalLabel(g.label) } : undefined;
}

/** The Byzantium `of`'s Heir: his position is public, so he is in every view, in fog too. */
function visibleHeir(view: PlayerView, of: PlayerId): UnitView | undefined {
  return view.units.find((u) => u.kind === 'Heir' && u.heir_of === of);
}

/** Where a held goal's item stands, when the viewer can place it: the Heir's tile (public every turn; `seen` says whether it is in sight now); the Hall's city while it is on the viewer's map (`seen` says whether it is in sight now). Public facts only. */
function heldPlace(view: PlayerView, of: PlayerId, faction: Faction): { pos: Hex; seen: boolean; city?: string; barbarian?: boolean } | undefined {
  if (faction === 'Byzantium') {
    const heir = visibleHeir(view, of);
    const t = heir && view.tiles[heir.pos.r * view.width + heir.pos.q];
    return heir ? { pos: heir.pos, seen: t?.visible ?? true, barbarian: isBarbarian(heir.owner) } : undefined;
  }
  if (faction === 'Mongols') {
    const hall = view.courier_halls?.find((h) => h.of === of);
    const city = hall && view.cities.find((c) => c.id === hall.city);
    return city ? { pos: city.pos, seen: city.visible, city: city.name } : undefined;
  }
  if (faction === 'India') {
    const diwan = view.imperial_diwans?.find((d) => d.of === of);
    const city = diwan && view.cities.find((c) => c.id === diwan.city);
    return city ? { pos: city.pos, seen: city.visible, city: city.name } : undefined;
  }
  return undefined;
}

/** Who holds a held goal's item and, when the viewer can place it, where: public facts only (the holders come from every empire's public VP sources). */
function heldHoldNote(view: PlayerView, faction: Faction, holders: PlayerId[], at: ReturnType<typeof heldPlace>, name: (id: PlayerId) => string): string {
  const names = holders.map(name).join(' and ');
  const verb = holders.includes(view.me) && holders.length === 1 ? 'hold' : 'holds';
  if (faction === 'Mongols') {
    if (!holders.length) return 'Nobody holds the Yam Relay Station: it is not built, its tile is unclaimed, or it was destroyed';
    return `${names} ${verb} the Yam Relay Station${at?.city ? ` in ${at.city}` : ''}${at ? (at.seen ? ' · in view' : ' · last seen') : ''}`;
  }
  if (faction === 'India') {
    if (!holders.length) return 'Nobody holds the Imperial Diwan yet';
    return `${names} ${verb} the Imperial Diwan${at?.city ? ` in ${at.city}` : ''}`;
  }
  const who = holders.length ? `${names} ${verb} the Heir` : at?.barbarian ? 'Barbarians hold the Heir: nobody scores until the camp is cleared' : 'Nobody holds the Heir now';
  return `${who}${at ? (at.seen ? ' · in view' : ' · out of sight') : ''}`;
}

/** Your own Heir goal in one line (the Byzantium has no count to show): who holds him now, or that nobody does. Null for any other faction. */
export function ownHeirLine(view: PlayerView): string | null {
  const me = view.players.find((p) => p.id === view.me);
  if (!me || me.faction !== 'Byzantium') return null;
  const holders = view.players.filter((p) => p.alive && p.vp_sources.some(([src]) => src.kind === 'FactionGoal' && src.of === me.id)).map((p) => p.id);
  const heir = visibleHeir(view, me.id);
  if (holders.length) return `The Heir: alive, held by ${holders.map((h) => (h === me.id ? 'you' : empireOfPlayer(view, h))).join(' and ')}`;
  if (heir && isBarbarian(heir.owner)) return 'The Heir: alive, held by barbarians (nobody scores)';
  return 'The Heir: nobody holds him (not trained, or lost)';
}

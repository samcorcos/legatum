// The two goals that live in a city (rules 51): Rome's Colonia Victrix and Egypt's Royal Tomb.
// Whoever holds the city scores its faction's goal VP, so the city is public to every empire, through
// fog and on unexplored land (core `view.colonias`, `view.royal_tombs`, like the Heir). One place
// gives every surface its mark, its name and its words: the banner end-cap, the tile and banner
// hovers, the attack forecast, the race row, the ledger, the city panel and the report.
// Rules live in the core (`goals.rs`); nothing here decides who scores.
import { RULES, factionGoalVp } from './data';
import { empireOfPlayer } from './empires';
import type { CityId, Faction, Hex, Order, PlayerId, PlayerPublic, PlayerView, UnitId } from './types';

export type GoalCityKind = 'colonia' | 'tomb';

/** One goal city as every empire sees it. */
export interface GoalCity {
  kind: GoalCityKind;
  /** The empire whose goal it is (Rome for a Colonia, Egypt for a tomb). */
  of: PlayerId;
  /** Who holds the city, and so scores the goal now. */
  holder: PlayerId;
  city: CityId;
  at: Hex;
  name: string;
}

/** The one mark per goal, everywhere it appears. */
export const GOAL_CITY_ICON: Record<GoalCityKind, string> = { colonia: 'goal-colonia', tomb: 'goal-royal-tomb' };
/** The goal's own name, capitalised as a title. */
export const GOAL_CITY_TITLE: Record<GoalCityKind, string> = { colonia: 'Colonia Victrix', tomb: 'Royal Tomb' };
/** The faction whose goal each kind is. */
export const GOAL_CITY_FACTION: Record<GoalCityKind, Faction> = { colonia: 'Rome', tomb: 'Egypt' };
export const goalCityKindOf = (faction: Faction): GoalCityKind | null => (faction === 'Rome' ? 'colonia' : faction === 'Egypt' ? 'tomb' : null);

/** Core `goals::COLONIA_LEVEL`: the level a founding veteran needs. */
export const COLONIA_LEVEL: number = RULES.coloniaLevel;
/** Core `goals::COLONIA_BORDER_REACH`: a site counts when a rival owns a tile this close. */
export const COLONIA_REACH: number = RULES.coloniaBorderReach;
/** Core `goals::TOMB_TURNS`: turns of work before the tomb stands. */
export const TOMB_TURNS: number = RULES.tombTurns;

const goalVp = (kind: GoalCityKind) => factionGoalVp(GOAL_CITY_FACTION[kind]);
const possessive = (name: string) => (name.endsWith('s') ? `${name}'` : `${name}'s`);
const tiles = (n: number) => `${n} ${n === 1 ? 'tile' : 'tiles'}`;

/** Every standing goal city in the view. */
export function goalCities(view: Pick<PlayerView, 'colonias' | 'royal_tombs'>): GoalCity[] {
  return [
    ...(view.colonias ?? []).map((g) => ({ kind: 'colonia' as const, ...g })),
    ...(view.royal_tombs ?? []).map((g) => ({ kind: 'tomb' as const, ...g })),
  ];
}

/** The goal cities that stand in one city (a city can carry both, if a captured Colonia became Egypt's capital). */
export function goalCitiesAt(view: Pick<PlayerView, 'colonias' | 'royal_tombs'>, city: CityId): GoalCity[] {
  return goalCities(view).filter((g) => g.city === city);
}

/** Where each Royal Tomb under construction will stand (core `goals::tomb_site`, public): one per empire whose work is on. */
export function tombSites(view: Pick<PlayerView, 'tomb_sites'>): GoalCity[] {
  return (view.tomb_sites ?? []).map((g) => ({ kind: 'tomb' as const, ...g }));
}

/** The tomb under construction that will stand in this city, if any. */
export function tombSiteAt(view: Pick<PlayerView, 'tomb_sites'>, city: CityId): GoalCity | undefined {
  return tombSites(view).find((g) => g.city === city);
}

/** A planned tomb's hover line: "Egypt's Royal Tomb will stand here in 9 turns: 2 VP to whoever holds the city.",
 *  "Your Royal Tomb will stand here in 9 turns.", or, paused, "Egypt's Royal Tomb will stand here: 2 VP to whoever holds the city. Paused at 6 of 15 turns." */
export function tombSiteLine(view: Pick<PlayerView, 'me' | 'players'>, g: Pick<GoalCity, 'of'>): string {
  const p = view.players.find((x) => x.id === g.of);
  const st = p ? tombStatus(p) : null;
  const whose = goalCityName(view, { kind: 'tomb', of: g.of });
  if (st && !st.working) return g.of === view.me ? `${whose}, paused at ${st.turns} of ${st.target} turns, will stand here.` : `${whose} will stand here: ${goalVp('tomb')} VP to whoever holds the city. Paused at ${st.turns} of ${st.target} turns.`;
  const when = st ? ` in ${st.left} ${st.left === 1 ? 'turn' : 'turns'}` : '';
  return g.of === view.me ? `${whose} will stand here${when}.` : `${whose} will stand here${when}: ${goalVp('tomb')} VP to whoever holds the city.`;
}

/** The seal a city's banner wears: a standing goal city (the tomb first if a city holds both), else a planned tomb, dashed. */
export function goalSealFor(view: Pick<PlayerView, 'colonias' | 'royal_tombs' | 'tomb_sites'>, city: CityId): { kind: GoalCityKind; of: PlayerId; dashed: boolean } | null {
  const standing = goalCitiesAt(view, city).sort((a, b) => Number(b.kind === 'tomb') - Number(a.kind === 'tomb'))[0];
  if (standing) return { kind: standing.kind, of: standing.of, dashed: false };
  const site = tombSiteAt(view, city);
  return site ? { kind: 'tomb', of: site.of, dashed: true } : null;
}


/** "Rome's Colonia Victrix", "Your Royal Tomb", "the Colonia Victrix" (lower-case start for mid-sentence). */
export function goalCityName(view: Pick<PlayerView, 'me' | 'players'>, g: Pick<GoalCity, 'kind' | 'of'>): string {
  return g.of === view.me ? `Your ${GOAL_CITY_TITLE[g.kind]}` : `${possessive(empireOfPlayer(view, g.of))} ${GOAL_CITY_TITLE[g.kind]}`;
}

type HoldView = Pick<PlayerView, 'me' | 'team' | 'players'>;
const teamOf = (view: HoldView, id: PlayerId) => view.players.find((p) => p.id === id)?.team;

/** The one line every hover and panel prints for a goal city: whose goal, who scores now, and what a rival can do. */
export function goalCityLine(view: HoldView, g: GoalCity): string {
  const vp = goalVp(g.kind), what = goalCityName(view, g);
  if (g.holder === view.me) return `${what}: you score ${vp} VP while you hold it.`;
  if (teamOf(view, g.holder) === view.team) return `${what}: your teammate scores ${vp} VP while holding it.`;
  const holder = empireOfPlayer(view, g.holder);
  return g.of === view.me
    ? `${what}: ${holder} scores its ${vp} VP while holding it. Retake the city to score them.`
    : `${what}: ${holder} scores ${vp} VP while holding it. Take the city to take the points.`;
}

/** The banner's hover part, in the order the banner draws it: "Colonia Victrix of Rome · whoever holds it scores 2 VP". */
export function goalCityBannerPart(view: Pick<PlayerView, 'players'>, g: GoalCity): string {
  return `${GOAL_CITY_TITLE[g.kind]} of ${empireOfPlayer(view, g.of)} · whoever holds it scores ${goalVp(g.kind)} VP`;
}

/** The goal points a capture of `city` by `captor` would take: one entry per goal city there that a rival team holds now. */
export function goalCityCaptures(view: Pick<PlayerView, 'players' | 'colonias' | 'royal_tombs'>, city: CityId, captor: PlayerId): { g: GoalCity; vp: number }[] {
  const captorTeam = view.players.find((p) => p.id === captor)?.team;
  return goalCitiesAt(view, city)
    .filter((g) => g.holder !== captor && view.players.find((p) => p.id === g.holder)?.team !== captorTeam)
    .map((g) => ({ g, vp: goalVp(g.kind) }));
}

/** The forecast's gilt line for a capture: "+2 VP while you hold Ostia: Rome's Colonia Victrix". */
export function goalCityCaptureLine(view: Pick<PlayerView, 'me' | 'players'>, g: GoalCity, vp: number): string {
  return `+${vp} VP while you hold ${g.name}: ${g.of === view.me ? `your ${GOAL_CITY_TITLE[g.kind]}` : `${possessive(empireOfPlayer(view, g.of))} ${GOAL_CITY_TITLE[g.kind]}`}`;
}

// ---- Egypt's Royal Tomb: the public work and its count ----

export interface TombStatus {
  /** The work is on: none of the empire's cities builds or grows. */
  working: boolean;
  /** Turns of work done (kept across a pause). */
  turns: number;
  target: number;
  /** Turns of work still to do. */
  left: number;
}

/** An empire's public tomb work, or null when it has none under way and none counted. */
export function tombStatus(p: Pick<PlayerPublic, 'tomb_work' | 'tomb_turns'>): TombStatus | null {
  const working = !!p.tomb_work, turns = Math.min(p.tomb_turns ?? 0, TOMB_TURNS);
  if (!working && turns === 0) return null;
  return { working, turns, target: TOMB_TURNS, left: Math.max(0, TOMB_TURNS - turns) };
}

/** The persistent state, short enough for the top bar and a race chip: "Royal Tomb · 6 of 15 turns". */
export const tombShort = (st: TombStatus): string => `Royal Tomb · ${st.turns} of ${st.target} turns`;
/** The race chip's count: "6/15". */
export const tombChip = (st: TombStatus): string => `${st.turns}/${st.target}`;

/** The public countdown as one line, for another empire's row and card (third person). */
export function tombPublicLine(st: TombStatus): string {
  return st.working
    ? `Building its Royal Tomb: ${st.turns} of ${st.target} turns. Its cities do not build or grow.`
    : `Royal Tomb paused at ${st.turns} of ${st.target} turns.`;
}

/** Egypt's own meter label, from the structured view (the core's label says the same in its own words). */
export function ownTombLabel(view: PlayerView): string | null {
  const t = view.actions?.tomb;
  if (!t) return null;
  const standing = goalCities(view).find((g) => g.kind === 'tomb' && g.of === view.me);
  if (standing) return standing.holder === view.me ? `The Royal Tomb stands in ${standing.name}` : `A rival holds the Royal Tomb in ${standing.name}`;
  return t.working ? `Building the Royal Tomb · ${t.turns} of ${t.target} turns` : `${t.turns} of ${t.target} turns of work on the Royal Tomb`;
}

/** The city the tomb would stand in now, named, for the confirm ("it will stand in Thebes"). */
export function tombSiteName(view: PlayerView): string | null {
  const id = view.actions?.tomb?.site;
  return id === null || id === undefined ? null : view.cities.find((c) => c.id === id)?.name ?? null;
}

/** Whether the viewer's empire is building its tomb now: every one of its cities is stopped. */
export function tombWorking(view: Pick<PlayerView, 'actions'>): boolean {
  return !!view.actions?.tomb?.working;
}

/** The Royal Tomb order queued this turn, if any: true builds, false pauses. */
export function pendingTomb(orders: readonly Order[]): boolean | null {
  const o = [...orders].reverse().find((x) => x.type === 'Action' && x.action.action === 'Tomb');
  return o && o.type === 'Action' && o.action.action === 'Tomb' ? o.action.build : null;
}

/** Whether the tomb work will be on at this turn's end, counting a Build or Pause order queued now: no city needs a build then. */
export function tombOnAfterOrders(view: Pick<PlayerView, 'actions'>, orders: readonly Order[]): boolean {
  return pendingTomb(orders) ?? tombWorking(view);
}

/** The city panel's and the overview's note on a stopped city. */
export const TOMB_CITY_NOTE = 'Working on the Royal Tomb: this city does not build or grow.';
/** The core's breakdown line for the stop (`economy.rs`): a yield row with this label is the tomb's. */
export const TOMB_BREAKDOWN_LABEL = 'The Royal Tomb: this city works on it';

/** Core tomb blockers mapped to a short phrase for a button's side; an unmapped one shows as the core wrote it. */
export function tombBlockerPhrase(blocker: string): string {
  if (blocker.startsWith('The Royal Tomb already stands in')) return blocker.replace('The Royal Tomb already stands in', 'It stands in');
  if (blocker === 'The Royal Tomb needs a city') return 'You need a city first.';
  return blocker;
}

// ---- Rome's Colonia Victrix: the founders and the sites ----

/** Whether this unit of yours founds the Colonia Victrix on a site that counts (a melee veteran at the level, while none stands). */
export function isColoniaFounder(view: Pick<PlayerView, 'actions'>, unit: UnitId): boolean {
  return !!view.actions?.colonia?.founders.includes(unit);
}

/** Rome's goal is open: no Colonia of its own stands (core `goals::known_colonia_open`). */
export function coloniaOpen(view: Pick<PlayerView, 'actions'>): boolean {
  return !!view.actions?.colonia;
}

/** What a site that counts is, in one line (the settle legend's key and the no-site note). */
export const COLONIA_SITE_RULE = `A site counts if it is within ${tiles(COLONIA_REACH)} of a rival's land.`;
/** The legend line for the gilt ring. */
export const COLONIA_RING_KEY = `Gilt ring: founds your Colonia Victrix (+${factionGoalVp('Rome')} VP)`;
/** The founding button on a site that counts. */
export const COLONIA_FOUND_LABEL = `Found Colonia Victrix · +${factionGoalVp('Rome')} VP`;
/** The hover's lead on a site that counts. */
export const COLONIA_SITE_LEAD = `Founding here makes your Colonia Victrix: +${factionGoalVp('Rome')} VP while you hold it.`;

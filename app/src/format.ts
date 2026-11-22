import { RESOURCE_INFO, fixClause, idleCountText, idleEntries, withArticle, withThe } from './resources';
import { WONDER_INFO, wonderBonusSentence } from './wonders';
import { diplomaticTeams, offerLabel } from './diplomacy';
import { ALLIANCE_SHARES, allianceTerms } from './treaties';
import { ruinText } from './celebrate/moments';
import { STOP_REASON } from './automation';
// Turn report: events -> human-readable lines, grouped and filterable. Battles are explained
// with the numbers that decided them (pillar 5: no hidden math).
import { improvementTitle } from './improvements';
import type { Breakdown, CityView, Event, Faction, Hex, PlayerId, PlayerView, ProductionItem, Resource, TechId, UnitKind, UpkeepPreview } from './types';
import { techCardUnlocks, techUnlockSummary } from './ui/techCardUnlocks';
import { eraName } from './ui/eraResearch';
import { cardName } from './ageCards';
import { CITY_HEAL, CITY_HP, cityCanFire } from './combat';
import { BUILDING_NAME, FACTION_ADJECTIVE, FACTION_GOAL, factionGoalVp, RULES, TECH_NAME, CAPTURE_RESISTANCE_TURNS, eraScaled, factionGoalRule, UNIT_NAME, itemName, pluralUnit } from './data';
import { closeObjectives } from './eraObjectives';
import { bearing, empireNameOf, empireOfPlayer, empireTitle, PEOPLE_NAME, placeName } from './empires';
import { CAMP_CLEAR_GOLD, isBarbarian } from './barbarians';
import { STRENGTH_PERCENT } from './promotion';
import { distance, hexEq } from './hex';
import { releasedGarrison } from './releasedGarrison';
import { sakokuTarget } from './sakoku';
import { cityHits, forceText, fromText } from './siege';
import { foundCityBlocker } from './settlement';
import { claimedTile, growthUnlocks } from './milestones';
import { eventIcon } from './reportIcons';
import { HOLD_TURNS, holdRule, turnsWord } from './victoryHold';
import { isAircraft, isFighter, readyNotice } from './airText';
import { wonderLostLine } from './wonderRace';

/** Core `MOVE_STOPPED_PREFIX`: the MoveBlocked reason that stops a move after repeated blocked turns. */
export const MOVE_STOPPED_PREFIX = 'move stopped after 2 blocked turns: ';
/** Core `FRIEND_BLOCK_PREFIX`: a step into a friend's tile that the friend did not leave; the rest names it. */
export const FRIEND_BLOCK_PREFIX = 'blocked by ';
/** Core `movement::TARGET_GONE`: an attack whose target died before movement, with no other enemy left on its tile. */
export const TARGET_GONE = 'its target was destroyed before it could attack';
/** Core `PEACE_BLOCK_PREFIX`: a move onto a unit of a team at Peace; the rest names that unit. */
export const PEACE_BLOCK_PREFIX = 'at Peace: ';

export type EventCategory = 'attack' | 'combat' | 'cities' | 'movement' | 'science' | 'victory' | 'economy' | 'orders' | 'diplomacy' | 'discoveries';

export const CATEGORY_LABEL: Record<EventCategory, string> = {
  attack: 'Under attack',
  diplomacy: 'Diplomacy',
  discoveries: 'Discoveries',
  victory: 'Victory race',
  combat: 'Combat',
  cities: 'Cities',
  science: 'Research',
  economy: 'Economy',
  movement: 'Movement',
  orders: 'Orders',
};

export const CATEGORY_ORDER: EventCategory[] = ['attack', 'diplomacy', 'victory', 'discoveries', 'combat', 'cities', 'science', 'economy', 'movement', 'orders'];

export interface ReportLine {
  category: EventCategory;
  text: string;
  /** Extra lines (battle math, reasons). */
  details: string[];
  at: Hex | null;
  /** Players this line concerns (for the Mine / Team / All filter). */
  players: PlayerId[];
  /** 0 = routine, 1 = notable, 2 = major. */
  weight: 0 | 1 | 2;
  /** From the viewer's side: bad (red), good (green) or neutral. Always shown with an icon too. */
  tone: Tone;
  openInboxTeam?: number;
  /** Your own finished technology: clicking the line shows its unlock card again. */
  reopenTech?: TechId;
  /** Your own era-up: clicking the line shows the takeover again (`era` is the 1-based `EraEntered.era`). */
  reopenEra?: { era: number; first: boolean; age?: PlayerId };
  /** A World Age began: the line opens the score ledger, where its card is. */
  openLedger?: boolean;
  /** A team that reached its VP target: clicking the line shows the alarm card again. */
  reopenHold?: HoldCard;
  /** The other empire (team) a diplomacy line is about, for its color mark. */
  empire?: number;
  /**
   * Combat: where the attacker stood, as far as the viewer saw it ("from where?"). The report
   * draws an arrow from here to `at` when the line is clicked.
   */
  from?: Hex | null;
  /** Signed HP changes written in `text`, in order, each coloured from the viewer's side. */
  hp?: HpMark[];
  /** The line's own medallion icon; the report falls back to its category's. */
  icon?: string;
  /** A newly earned promotion, shown with the same insignia as its unit. */
  promotion?: { level: number; faction?: Faction };
}

/** What the victory-hold card needs about one team's hold: the event's own numbers. */
export interface HoldCard { team: number; vp: number; target: number; turnsNeeded: number; /** True for the near-victory warning (two VP short) rather than a started countdown. */ near?: boolean }

export type Tone = 'good' | 'bad' | 'neutral';
/** A signed HP change as it appears in a line's text ("−35 HP"). */
interface HpMark {
  text: string;
  tone: Tone;
}

/** Names the formatter needs; built from the views before and after the turn. */
export interface ReportContext {
  me: PlayerId;
  team: PlayerId[];
  playerName(id: PlayerId | null): string;
  playerFaction?(id: PlayerId): Faction | undefined;
  /** An era's name from the core ("Consolidation"), for the era events. */
  eraName?(era: number): string | undefined;
  /** World Ages: the rule behind one of an Age's cards (undefined in a game without them). */
  ageCard?(era: number, slot: number): { kind: string; [field: string]: unknown } | undefined;
  /** The buildings and units your own finished technology unlocked, in your faction's names. */
  techUnlocks?(tech: TechId): string[];
  teamName?(team: number): string;
  teamId?: number;
  teamPlayers?(team: number): PlayerId[];
  /** The empire's name as everywhere else in the UI (`empireTitle`). */
  empireName?(team: number): string;
  /** One player's empire as everywhere else in the UI (`empireOfPlayer`); falls back to `playerName`. */
  empireOf?(id: PlayerId): string;
  /** Rewrites raw player names inside core's text ("AI 3's Scout") the same way. */
  renamePlayers?(text: string): string;
  /** Rewrites a leading raw player subject in core reasons ("Player 1 was eliminated"). */
  renameSubject?(text: string): string;
  /** A tile by the nearest known city ("2 tiles NE of Brasslock"), or null when none is near. */
  place?(hex: Hex): string | null;
  treatyKind?(id: number): 'Alliance' | null;
  /** Whether the viewer's team has an Alliance with this team (before or after the turn). */
  allied?(team: number): boolean;
  /** Wars still open after a pair signs Peace. */
  peaceContinuing?(a: number, b: number): [number, number][];
  events?: Event[];
  unitKind(id: number): UnitKind | null;
  /** The turn these events resolved (a War that starts in it was a surprise declaration, G20). */
  turn?: number;
  /** Whether the viewer knows a barbarian camp on this tile. */
  campAt?(hex: Hex): boolean;
  /** Where the viewer last saw one of its units (before or after the turn), for naming it by place. */
  unitPos?(id: number): Hex | null;
  cityName(id: number): string;
  cityPos(id: number): Hex | null;
  /** A city's HP at the start of the turn that resolved, when the view before it is known. */
  cityHpBefore?(id: number): number | null;
  /** A known city's tile by its name, for core reasons that name the city ("Hymnal can no longer…"). */
  cityNamed?(name: string): Hex | null;
  /** Where one of your idle buildings or units short of this resource is (the first in the supply's list), for "Show on map". */
  idleSupplyAt?(resource: Resource): Hex | null;
  /** Names of the idle users, including aircraft in hangars, for the shortage report. */
  idleSupplyNames?(resource: Resource): string[];
  /** What a city's new population unlocked ("Settler"), from the post-turn view's forecasts. */
  growthUnlocks?(city: number, population: number): string | null;
  /** A claimed border tile's terrain and yields as its owner works it, when the viewer knows it. */
  claimedTile?(tile: Hex, owner: PlayerId): { terrain: string; yields: string | null } | null;
}

export function contextFromViews(before: PlayerView | null, after: PlayerView): ReportContext {
  const players = new Map(after.players.map((p) => [p.id, p]));
  const units = new Map<number, UnitKind>();
  const unitPos = new Map<number, Hex>();
  const cities = new Map<number, { name: string; pos: Hex }>();
  const treaties = new Map<number, 'Alliance'>();
  for (const v of [before, after]) {
    if (!v) continue;
    for (const u of v.units) { units.set(u.id, u.kind); unitPos.set(u.id, u.pos); }
    for (const a of v.aircraft ?? []) units.set(a.id, a.kind);
    for (const c of v.cities) cities.set(c.id, { name: c.name, pos: c.pos });
    for (const r of v.diplomacy?.relations ?? []) {
      if (r.security && r.state === 'Alliance') treaties.set(r.security.id, r.state);
    }
  }
  const team = after.players.filter((p) => p.team === after.team).map((p) => p.id);
  return {
    me: after.me,
    playerFaction: (id) => players.get(id)?.faction,
    eraName: (era) => after.eras?.find((x) => x.era === era)?.name,
    ageCard: (era, slot) => after.eras?.find((x) => x.era === era)?.objectives.find((o) => o.slot === slot)?.objective,
    techUnlocks: (tech) => techCardUnlocks(after.research, tech, players.get(after.me)?.faction ?? 'India').filter((u) => u.kind !== 'Effect').map(techUnlockSummary),
    team,
    teamId: after.team,
    teamPlayers: (team) => after.players.filter((p) => p.team === team).map((p) => p.id),
    empireName: (team) => empireTitle(after, team),
    treatyKind: (id) => treaties.get(id) ?? null,
    allied: (t) => [before, after].some((v) => v?.diplomacy?.relations.some((r) =>
      r.state === 'Alliance' && ((r.a === after.team && r.b === t) || (r.b === after.team && r.a === t)))),
    peaceContinuing: (a, b) => (after.diplomacy?.relations ?? [])
      .filter((r) => r.state === 'War' && (r.a === a || r.b === a || r.a === b || r.b === b)
        && !((r.a === a && r.b === b) || (r.a === b && r.b === a)))
      .map((r): [number, number] => [r.a, r.b]),
    teamName: (team) => (team === after.team ? 'you' : empireTitle(after, team)),
    playerName: (id) => (id === null ? 'nobody' : isBarbarian(id) ? 'Barbarians' : (players.get(id)?.name ?? `Empire ${id}`)),
    empireOf: (id) => (isBarbarian(id) ? 'Barbarians' : empireOfPlayer(after, id)),
    renamePlayers: (text) => renamePlayers(after, text),
    renameSubject: (text) => renameSubject(after, text),
    place: (hex) => placeName(after, hex, Infinity),
    unitKind: (id) => units.get(id) ?? null,
    turn: before?.turn ?? after.turn - 1,
    campAt: (hex) => [before, after].some((v) => v?.barbarian_camps?.some((c) => hexEq(c.pos, hex))),
    unitPos: (id) => unitPos.get(id) ?? null,
    cityName: (id) => cities.get(id)?.name ?? `a city`,
    cityPos: (id) => cities.get(id)?.pos ?? null,
    cityHpBefore: (id) => before?.cities.find((c) => c.id === id)?.hp ?? null,
    cityNamed: (name) => [...cities.values()].find((c) => c.name === name)?.pos ?? null,
    idleSupplyAt: (resource) => {
      const supply = after.supply?.find((x) => x.resource === resource);
      return supply ? idleEntries(after, supply).find((x) => x.hex)?.hex ?? null : null;
    },
    idleSupplyNames: (resource) => {
      const supply = after.supply?.find((x) => x.resource === resource);
      return supply ? idleEntries(after, supply).map((x) => x.label) : [];
    },
    growthUnlocks: (city, population) => growthUnlocks(after, city, population),
    claimedTile: (tile, owner) => claimedTile(after, tile, players.get(owner)?.faction),
  };
}

export function formatBreakdown(b: Breakdown): string {
  // A true minus for a negative total (a wounded Scout can fight at -1).
  const total = b.total < 0 ? `−${Math.abs(b.total)}` : String(b.total);
  if (!b.parts.length) return total;
  const parts = b.parts.map(([l, v], i) => (i === 0 ? `${l} ${v}` : `${l} ${v >= 0 ? '+' : '−'}${Math.abs(v)}`));
  return `${total} (${parts.join(', ')})`;
}

/** "Switzerland'" and "Celts's". */
const possessiveOf = (name: string) => (name.endsWith('s') ? `${name}'` : `${name}'s`);
/** A player named as everywhere else in the UI: "You", or their empire, never a raw seat name. */
const who = (ctx: ReportContext, id: PlayerId | null) =>
  id === null ? 'nobody' : id === ctx.me ? 'You' : (ctx.empireOf?.(id) ?? ctx.playerName(id));
/** The same, as an object ("from you"). */
const whom = (ctx: ReportContext, id: PlayerId | null) => (id === ctx.me ? 'you' : who(ctx, id));
const possessive = (ctx: ReportContext, id: PlayerId) => (isBarbarian(id) ? 'Barbarian' : id === ctx.me ? 'Your' : possessiveOf(who(ctx, id)));
const cap = (text: string) => text.charAt(0).toUpperCase() + text.slice(1);
/** Where it happened, by the nearest known city, or the map target when none is known. */
const at = (ctx: ReportContext, hex: Hex) => {
  const place = ctx.place?.(hex);
  return place ? (place.includes(' of ') ? place : `at ${place}`) : 'at this tile';
};
const location = (ctx: ReportContext, hex: Hex) => ctx.place?.(hex) ?? 'this tile';
const escapeRegExp = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/** Core writes unit owners by player name ("blocked by AI 3's Scout"); say "your" or the empire instead. */
function renamePlayers(view: PlayerView, text: string): string {
  return [...view.players].sort((a, b) => b.name.length - a.name.length).reduce((out, p) => out.replace(
    new RegExp(`(^|[^\\w])${escapeRegExp(p.name)}'s `, 'g'),
    (_, before: string) => `${before}${p.id === view.me ? 'your' : possessiveOf(empireOfPlayer(view, p.id))} `,
  ), text);
}

/** Use the viewer's empire names only when a core reason's subject uniquely identifies a player. */
function renameSubject(view: PlayerView, text: string): string {
  for (const p of [...view.players].sort((a, b) => b.name.length - a.name.length)) {
    const subject = new RegExp(`^${escapeRegExp(p.name)} (was|were|could not) `);
    if (!subject.test(text)) continue;
    if (view.players.filter((other) => other.name === p.name).length !== 1) return text;
    return text.replace(subject, (_, verb: string) =>
      p.id === view.me ? `you ${verb === 'could not' ? verb : 'were'} ` : `${empireNameOf([p], view.players)} ${verb} `);
  }
  return text;
}

function unitName(ctx: ReportContext, id: number, owner: PlayerId, kind?: UnitKind | null) {
  const k = kind ?? ctx.unitKind(id);
  return k ? itemName({ kind: 'Unit', unit: k }, ctx.playerFaction?.(owner)) : 'unit';
}

function unitLabel(ctx: ReportContext, id: number, owner: PlayerId, kind?: UnitKind | null) {
  return `${possessive(ctx, owner)} ${unitName(ctx, id, owner, kind)}`;
}

function line(
  category: EventCategory,
  text: string,
  opts: Partial<Omit<ReportLine, 'category' | 'text'>> = {},
): ReportLine {
  return { category, text, details: [], at: null, players: [], weight: 0, tone: 'neutral', ...opts };
}

/** Detail for pursuit lines: attack orders last one turn (G18). */
const ONE_TURN_ATTACK = 'An attack order lasts one turn and makes one attack. Order it again to attack next turn.';

/** Where a battle happened, naming a known barbarian camp on the tile. */
function battleWhere(ctx: ReportContext, hex: Hex): string {
  if (!ctx.campAt?.(hex)) return at(ctx, hex);
  const place = ctx.place?.(hex);
  return place ? `at the camp ${place.includes(' of ') ? place : `near ${place}`}` : 'at this camp';
}

type Battle = Extract<Event, { type: 'Battle' }>;
type Mine = (p: PlayerId) => boolean;

/** A signed HP loss as written in a line ("−35 HP"): bad when it is the viewer's, good when the viewer dealt it. */
function hpLoss(amount: number, owner: PlayerId, other: PlayerId, mine: Mine): HpMark {
  return { text: `−${amount} HP`, tone: mine(owner) ? 'bad' : mine(other) ? 'good' : 'neutral' };
}

/**
 * "From where?": the attacker's tile relative to its target ("from 2 tiles NE"). A city is named
 * by the line itself. An enemy that fired from tiles the viewer never saw is "out of sight".
 */
function battleFrom(e: Battle, mine: Mine): string {
  if (typeof e.attacker !== 'number') return '';
  if (!e.attacker_at) return mine(e.attacker_owner) ? '' : ' from out of sight';
  const n = distance(e.at, e.attacker_at);
  return n === 0 ? '' : ` from ${n} tile${n === 1 ? '' : 's'} ${bearing(e.at, e.attacker_at)}`;
}

/**
 * A battle from the viewer's side. Your attack that hurt or killed is good; your unit or city hurt
 * is bad; a death decides it whoever started; in a meeting both were moving, so the bigger loss
 * decides. Other empires' battles are neutral.
 */
/** Melee battles grouped by defender, kept only where several attackers hit it this turn (split counterattack). */
function combinedAssaults(events: Event[]): Map<string, Battle[]> {
  const groups = new Map<string, Battle[]>();
  for (const e of events) if (e.type === 'Battle' && !e.ranged && !e.meeting && typeof e.attacker === 'number') {
    const key = assaultKey(e.defender);
    groups.set(key, [...(groups.get(key) ?? []), e]);
  }
  for (const [key, group] of groups) if (group.length < 2) groups.delete(key);
  return groups;
}
const assaultKey = (d: Battle['defender']) => (d.kind === 'City' ? `City:${d.city}` : `Unit:${d.unit}`);

/** "your 3 Oathbearers", "Switzerland' 2 Soldiers and Trooper": who joined a combined assault. */
function assaultForce(ctx: ReportContext, fights: Battle[]): string {
  const counts = new Map<string, number>();
  for (const b of fights) {
    const name = unitName(ctx, b.attacker as number, b.attacker_owner);
    counts.set(name, (counts.get(name) ?? 0) + 1);
  }
  const parts = [...counts].map(([name, n]) => (n === 1 ? name : `${n} ${pluralUnit(name)}`));
  const kinds = parts.length === 1 ? parts[0] : `${parts.slice(0, -1).join(', ')} and ${parts.at(-1)}`;
  const owners = new Set(fights.map((b) => b.attacker_owner));
  const owner = owners.size === 1 ? fights[0].attacker_owner : null;
  const whose = owner === null ? '' : owner === ctx.me ? 'your ' : isBarbarian(owner) ? 'barbarian ' : `${possessiveOf(who(ctx, owner))} `;
  return `${whose}${kinds}`;
}

/** "−7 each", or "−8, −7, −7" when the strongest took the remainder. */
function counterSplit(fights: Battle[]): string {
  const taken = fights.map((b) => b.damage_to_attacker).sort((a, b) => b - a);
  return taken.every((n) => n === taken[0]) ? `−${taken[0]} each` : taken.map((n) => `−${n}`).join(', ');
}

function battleTone(e: Battle, mine: Mine): Tone {
  const atkMine = mine(e.attacker_owner), defMine = mine(e.defender.owner);
  if (atkMine === defMine) return 'neutral';
  const [mineHp, theirHp] = atkMine ? [e.attacker_hp_after, e.defender_hp_after] : [e.defender_hp_after, e.attacker_hp_after];
  if (mineHp <= 0) return 'bad';
  if (theirHp <= 0) return 'good';
  if (e.meeting) {
    const [lost, dealt] = atkMine ? [e.damage_to_attacker, e.damage_to_defender] : [e.damage_to_defender, e.damage_to_attacker];
    return dealt >= lost ? 'good' : 'bad';
  }
  return atkMine ? 'good' : 'bad';
}

/** How a city's automatic shot works, for its report line and the forecast. */
export const CITY_STRIKE_RULE = 'Walled cities fire back: each city with Walls and HP left shoots once per turn, before movement, at the weakest (lowest-HP) visible enemy military unit within 2 tiles. It deals damage but takes none. Cities without Walls cannot fire.';

/** How a city's HP comes back, for its health tooltip, the Guide and the attack forecast. */
/** The rule behind a city's strength number, with the walls it has (one wording for the city panel and the Cities overview). */
export function cityStrengthRule(city: Pick<CityView, 'walls' | 'citadel' | 'star_fort'>, faction?: Faction, era = 0): string {
  const at = (design: number) => eraScaled(design, era);
  return `City strength: ${at(10)} + ${at(2)} × population${city.walls ? (faction === 'Switzerland' ? ` + ${at(15)} Alpine Redoubt` : ` + ${at(10)} Walls`) : ''}${city.citadel ? ` + ${at(5)} Castle upgrade` : ''}${city.star_fort ? ` + ${at(5)} Star Fort upgrade` : ''}, less damage. Every part grows with its owner's era. Hits land one at a time. The hit that empties the city's HP carries on to the garrison with the part of its damage the city did not absorb. Later hits strike the garrison in full.`;
}

export const CITY_HEAL_RULE = `A city regains ${CITY_HEAL} HP at the end of each turn, unless it took damage that turn or an enemy military unit is adjacent. Empty to full takes ${CITY_HP / CITY_HEAL} turns.`;

/**
 * Your own damaged city's healing, for the city card: how much it regains and how soon it is
 * full, or why it will not heal this turn (core `CityDetail.heal_blocked`). Null at full HP or
 * without detail (a rival's city shows only the rule).
 */
export function cityHealStatus(city: CityView): { blocked: boolean; text: string } | null {
  const d = city.detail;
  if (!d || city.hp >= city.max_hp) return null;
  if (d.heal_blocked) return { blocked: true, text: 'Not healing: an enemy military unit is adjacent' };
  const turns = d.turns_to_heal ?? Math.ceil((city.max_hp - city.hp) / CITY_HEAL);
  return { blocked: false, text: `Heals +${CITY_HEAL} a turn · full ${turns <= 1 ? 'next turn' : `in ${turns} turns`}` };
}

/**
 * Whether a known city fires (core `forecast::city_fires`: Walls and HP left), in words for the
 * city panel, tile card and attack forecast.
 */
export function cityFireStatus(view: PlayerView, city: CityView): { fires: boolean; text: string } {
  const wall = view.players.find((p) => p.id === city.owner)?.faction === 'Switzerland' ? 'Alpine Redoubt' : 'Walls';
  if (!city.walls) return { fires: false, text: `No ${wall}: can't fire` };
  if (!cityCanFire(city.walls, city.hp)) return { fires: false, text: "0 HP: can't fire" };
  return { fires: true, text: `${wall}: fires at units within 2 tiles` };
}

/** Each empire's first 2 military units are free; every extra costs 1 Gold/turn, 3 for the
 * Switzerland. Scouts and civilians never cost upkeep and never use a free slot. One rule, one wording: the production picker, unit panel
 * and Guide all describe upkeep this way. */
const FREE_MILITARY_UNITS = 2;

/** A production card's upkeep line, for a unit not yet built (legatum-ui: "every build card
 * shows its ongoing cost"). Undefined `upkeep` means a Scout or civilian (Settlers included): no upkeep. */
export function upkeepText(upkeep?: UpkeepPreview | null): string {
  if (!upkeep) return 'No upkeep';
  if (upkeep.free) return `Upkeep: free · ${upkeep.free_slots_left} of ${FREE_MILITARY_UNITS} slots left`;
  return `Upkeep: −${upkeep.rate} Gold/turn`;
}

/** A city's automatic shot (cities fire back): "Levy Reach fired on Switzerland' Soldier at …: −14 HP". */
function cityShotLine(e: Battle, city: number, ctx: ReportContext, mine: Mine): ReportLine {
  const name = ctx.cityName(city);
  const shooter = e.attacker_owner === ctx.me ? name : `${possessiveOf(who(ctx, e.attacker_owner))} city ${name}`;
  const d = e.defender;
  const target = d.kind === 'Unit'
    ? unitLabel(ctx, d.unit, d.owner, d.unit_kind).replace(/^Your /, 'your ')
    : `city ${ctx.cityName(d.city)}`;
  const where = ctx.place?.(e.at) ? ` ${at(ctx, e.at)}` : '';
  const hp = [hpLoss(e.damage_to_defender, d.owner, e.attacker_owner, mine)];
  const text = e.defender_hp_after <= 0
    ? `${shooter} fired on ${target}${where}: ${hp[0].text}, destroyed`
    : `${shooter} fired on ${target}${where}: ${hp[0].text} (${e.defender_hp_after} left)`;
  return line('combat', cap(text), {
    at: e.at,
    from: e.attacker_at ?? ctx.cityPos(city),
    hp,
    players: [e.attacker_owner, d.owner],
    details: [
      `Strength ${formatBreakdown(e.attacker_strength)} vs ${formatBreakdown(e.defender_strength)}`,
      `Dealt ${e.damage_to_defender} damage (roll ${e.attacker_roll}%). No retaliation.`,
      CITY_STRIKE_RULE,
    ],
    weight: mine(e.attacker_owner) || mine(d.owner) ? 2 : 1,
    tone: battleTone(e, mine),
  });
}

/**
 * One battle as a report line that answers "who hit what, where, from where, and what it cost":
 * "Hollow Crown's Archer bombarded your Ranger at Beacon from 2 tiles NE: −35 HP (65 left)".
 * Melee lines also carry what happens next (G18, G21): an attack whose defender survived ends the
 * attacker's order ("Needs orders"), and a unit hit while pulling back either got away or was stopped.
 */
/**
 * A Fighter on guard met an air strike (core `air::resolve_ranged_phase`): the strike did not land and the two
 * aircraft traded damage. Aircraft never stand on the map, so this is the only report of the meeting. A rival's
 * hangar is hidden, so the striker's kind may be unknown ("an air strike").
 */
function interceptionLine(e: Battle, ctx: ReportContext, mine: Mine): ReportLine {
  const d = e.defender as Extract<Battle['defender'], { kind: 'Unit' }>;
  const attacker = e.attacker as number;
  const fighterOwner = d.owner, strikerOwner = e.attacker_owner;
  const city = ctx.place?.(e.at);
  const home = city && !city.includes(' of ') ? ` from ${city}` : '';
  const kind = ctx.unitKind(attacker);
  const striker = (owner: PlayerId) => (kind ? `${unitLabel(ctx, attacker, owner, kind)}` : `${possessive(ctx, owner)} air strike`);
  const lowerStriker = (owner: PlayerId) => striker(owner).replace(/^Your /, 'your ');
  const fighter = (owner: PlayerId) => `${owner === ctx.me ? 'your' : possessiveOf(who(ctx, owner))} Fighter${home}`;
  const toFighter = hpLoss(e.damage_to_defender, fighterOwner, strikerOwner, mine);
  const toStriker = hpLoss(e.damage_to_attacker, strikerOwner, fighterOwner, mine);
  const lost = e.defender_hp_after <= 0 ? ` The Fighter was shot down.` : e.attacker_hp_after <= 0 ? ` The ${kind ? UNIT_NAME[kind] : 'aircraft'} was shot down.` : '';
  const text = mine(fighterOwner)
    ? `${cap(fighter(fighterOwner))} intercepted ${lowerStriker(strikerOwner).replace(/^your /, '')}: the strike did not land.${lost}`
    : mine(strikerOwner)
      ? `${cap(lowerStriker(strikerOwner))} was intercepted by ${fighter(fighterOwner)}: the strike did not land.${lost}`
      : `${cap(lowerStriker(strikerOwner))} was intercepted by ${fighter(fighterOwner)}.${lost}`;
  const tone: Tone = mine(fighterOwner) === mine(strikerOwner) ? 'neutral' : mine(fighterOwner) ? 'good' : 'bad';
  return line('combat', text, {
    at: e.at, players: [strikerOwner, fighterOwner], weight: mine(fighterOwner) || mine(strikerOwner) ? 2 : 1, tone, hp: [toFighter, toStriker],
    details: [`The Fighter and the striking aircraft traded damage: Fighter ${toFighter.text}, striker ${toStriker.text}.`,
      `Strength ${formatBreakdown(e.attacker_strength)} vs ${formatBreakdown(e.defender_strength)}`,
      'A Fighter on guard meets one air strike a turn within its range; only a Stealth Fighter can meet a Stealth Bomber. A Fighter that strikes does not intercept.'],
  });
}

function battleLine(e: Battle, ctx: ReportContext, mine: Mine): ReportLine {
  if (typeof e.attacker !== 'number') return cityShotLine(e, e.attacker.city, ctx, mine);
  if (e.defender.kind === 'Unit' && isFighter(e.defender.unit_kind)) return interceptionLine(e, ctx, mine);
  const d = e.defender;
  const atk = e.air_strike && !ctx.unitKind(e.attacker)
    ? `${possessive(ctx, e.attacker_owner)} air strike`
    : unitLabel(ctx, e.attacker, e.attacker_owner);
  const lower = (s: string) => s.replace(/^Your /, 'your ');
  const defName = d.kind === 'Unit'
    ? lower(unitLabel(ctx, d.unit, d.owner, d.unit_kind))
    : `${d.owner === ctx.me ? 'your' : possessiveOf(who(ctx, d.owner))} city ${ctx.cityName(d.city)}`;
  const def = d.kind === 'Unit' && isBarbarian(d.owner) ? `the ${defName}` : defName;
  const from = battleFrom(e, mine);
  const where = battleWhere(ctx, e.at);
  const toDef = hpLoss(e.damage_to_defender, d.owner, e.attacker_owner, mine);
  const toAtk = hpLoss(e.damage_to_attacker, e.attacker_owner, d.owner, mine);
  const details = [`Strength ${formatBreakdown(e.attacker_strength)} vs ${formatBreakdown(e.defender_strength)}`,
    e.ranged
      ? `Dealt ${e.damage_to_defender} damage (roll ${e.attacker_roll}%). Ranged: no retaliation.`
      : `Dealt ${e.damage_to_defender} (roll ${e.attacker_roll}%), took ${e.damage_to_attacker} (roll ${e.defender_roll}%).`];
  if (e.meeting) details.push('Meeting engagement: both were moving, so no terrain or fortify bonus.');
  const involved = mine(e.attacker_owner) || mine(d.owner);
  const base = {
    at: e.at, from: e.attacker_at ?? null, players: [e.attacker_owner, d.owner],
    weight: involved ? 2 as const : 1 as const, tone: battleTone(e, mine), details,
  };
  const both = e.attacker_hp_after > 0 && e.defender_hp_after > 0;

  if (e.ranged) {
    const text = e.defender_hp_after <= 0
      ? `${atk} bombarded ${def} ${where}${from && `,${from}`}: ${toDef.text}, ${d.kind === 'City' ? 'the city fell to 0 HP' : 'destroyed'}`
      : `${atk} bombarded ${def} ${where}${from && `,${from}`}: ${toDef.text} (${e.defender_hp_after} left)`;
    return line('combat', text, { ...base, hp: [toDef] });
  }

  // G21: a melee hit on a unit pulling back. A survivor completes its move.
  if (both && e.defender_leaving && d.kind === 'Unit' && involved) {
    const events = ctx.events ?? [];
    const reached = events.slice(events.indexOf(e) + 1)
      .flatMap((x) => (x.type === 'UnitMoved' && x.unit === d.unit && x.to ? [x.to] : [])).at(-1) ?? null;
    const place = reached ? (ctx.place?.(reached) ?? 'this tile') : null;
    details.push('Hit while pulling back: no fortification bonus. A unit that survives completes its move.');
    if (mine(d.owner)) {
      return line('combat', `${cap(defName)} was hit by ${lower(atk)}${from} while pulling back (${toDef.text}) and ${place ? `reached ${place}` : 'was stopped'}`, {
        ...base, at: reached ?? e.at, hp: [toDef],
      });
    }
    const name = unitName(ctx, d.unit, d.owner, d.unit_kind);
    return line('combat', `${atk} hit the retreating ${name}${from} (${toDef.text}); it ${reached ? 'got away' : 'was stopped'} with ${e.defender_hp_after} HP. Needs orders.`, {
      ...base, hp: [toDef],
    });
  }

  const verb = e.meeting ? 'clashed with' : 'attacked';
  const losses = `defender ${toDef.text}${e.damage_to_attacker > 0 ? `, attacker ${toAtk.text}` : ''}`;
  const hp = e.damage_to_attacker > 0 ? [toDef, toAtk] : [toDef];
  if (both) {
    // One attack per order (G18): whoever attacked and left its opponent standing needs orders.
    const mineLeft = mine(e.attacker_owner) ? e.attacker_hp_after : e.defender_hp_after;
    const next = mine(e.attacker_owner) || (e.meeting && mine(d.owner)) ? ' Needs orders.' : '';
    const left = involved ? ` (${mineLeft} HP left)` : ` (${e.attacker_hp_after} vs ${e.defender_hp_after} HP left)`;
    return line('combat', `${atk} ${verb} ${def} ${where}${from && `,${from}`}: ${losses}${left}.${next}`, { ...base, hp });
  }
  const outcome = e.defender_hp_after <= 0
    ? d.kind === 'City' ? 'The city fell to 0 HP.' : 'The defender was destroyed.'
    : 'The attacker was destroyed.';
  return line('combat', `${atk} ${verb} ${def} ${where}${from && `,${from}`}: ${losses}. ${outcome}`, { ...base, hp });
}

type GoalEvent = Extract<Event, { type: 'FactionGoalCompleted' | 'FactionGoalLost' }>;

/** The Heir changes hands in one pair of events: the old holder's loss comes with the new holder's gain. */
const heirLoss = (events: Event[] | undefined) =>
  events?.find((x): x is Extract<Event, { type: 'FactionGoalLost' }> => x.type === 'FactionGoalLost' && x.faction === 'Byzantium');

/**
 * The Byzantium's held goal, in the viewer's words. A hand-over is one line from the new holder's
 * event (the old holder's loss folds into it, see `formatEvents`); a loss with no new holder (he
 * was disbanded, killed, or taken by barbarians) is its own line.
 */
function heirGoalLine(e: GoalEvent, ctx: ReportContext): ReportLine {
  const mine = (p: PlayerId) => p === ctx.me || ctx.team.includes(p);
  const isByzantium = (p: PlayerId) => ctx.playerFaction?.(p) === 'Byzantium';
  const subject = (p: PlayerId) => (p === ctx.me ? 'You' : who(ctx, p));
  const gain = (p: PlayerId) => (mine(p) ? ` (+${e.vp} VP)` : '');
  const loss = (p: PlayerId) => (mine(p) ? ` (−${e.vp} VP)` : '');
  const base = { weight: 2, icon: 'unit-heir' } as const;
  if (e.type === 'FactionGoalLost') {
    const text = isByzantium(e.player) ? `${subject(e.player)} lost the Heir: nobody scores until he is held again` : `${subject(e.player)} lost Byzantine Heir`;
    return line('victory', `${text}${loss(e.player)}`, { ...base, players: [e.player], tone: mine(e.player) ? 'bad' : 'neutral' });
  }
  const taker = e.player;
  const from = heirLoss(ctx.events)?.player ?? null;
  const opts = { ...base, players: from === null ? [taker] : [taker, from] };
  if (isByzantium(taker)) {
    if (from === null) return line('victory', `${subject(taker)} trained the Heir${gain(taker)}`, { ...opts, tone: mine(taker) ? 'good' : 'neutral' });
    const text = `${subject(taker)} retook ${taker === ctx.me ? 'your' : 'its'} Heir from ${whom(ctx, from)}`;
    return line('victory', `${text}${gain(taker)}${loss(from)}`, { ...opts, tone: mine(taker) ? 'good' : mine(from) ? 'bad' : 'neutral' });
  }
  if (from === null) return line('victory', `${subject(taker)} freed Byzantium's Heir${gain(taker)}`, { ...opts, tone: mine(taker) ? 'good' : 'neutral' });
  if (mine(taker)) return line('victory', `${subject(taker)} captured Byzantium's Heir from ${whom(ctx, from)}${gain(taker)}`, { ...opts, tone: 'good' });
  if (mine(from)) {
    const what = isByzantium(from) && from === ctx.me ? 'your Heir' : `Byzantium's Heir from ${whom(ctx, from)}`;
    return line('victory', `${subject(taker)} captured ${what}${loss(from)}`, { ...opts, tone: 'bad' });
  }
  return line('victory', `Byzantium's Heir changed hands: ${who(ctx, taker)} now holds him`, { ...opts, tone: 'neutral' });
}

/**
 * Mongols's held goal: the Courier Hall. A capture is one line from the new holder's gain, with the
 * old holder's loss folded in (see `formatEvents`); a Hall built has no loss beside it; a loss with
 * no new holder is its city sacked (or taken by barbarians). The city's name comes from the capture
 * or completion event when the viewer saw one.
 */
function hallGoalLine(e: GoalEvent, ctx: ReportContext): ReportLine {
  const mine = (p: PlayerId) => p === ctx.me || ctx.team.includes(p);
  const subject = (p: PlayerId) => (p === ctx.me ? 'You' : who(ctx, p));
  const events = ctx.events ?? [];
  const built = (p: PlayerId) => events.find((x): x is Extract<Event, { type: 'ProductionCompleted' }> => x.type === 'ProductionCompleted' && x.owner === p && (x.item.kind === 'Building' || x.item.kind === 'Wonder') && x.item.building === 'CourierHall');
  const base = { weight: 2, icon: 'building-courierhall' } as const;
  if (e.type === 'FactionGoalLost') {
    const razed = events.find((x): x is Extract<Event, { type: 'CityRazed' }> => x.type === 'CityRazed' && x.from === e.player);
    const lost = mine(e.player) ? ` (−${e.vp} VP)` : '';
    const text = razed
      ? `${e.player === ctx.me ? 'Your' : `${possessiveOf(who(ctx, e.player))}`} Yam Relay Station is gone: its city was sacked${mine(e.player) ? ', and the open road with it. It can be built again' : ''}`
      : `${subject(e.player)} lost the Yam Relay Station: nobody scores until its city is retaken`;
    return line('victory', `${text}${lost}`, { ...base, players: [e.player], at: razed?.at, tone: mine(e.player) ? 'bad' : 'neutral' });
  }
  const taker = e.player;
  const gain = mine(taker) ? ` (+${e.vp} VP)` : '';
  const cap = events.find((x): x is Extract<Event, { type: 'CityCaptured' }> => x.type === 'CityCaptured' && x.to === taker);
  const city = cap?.city ?? built(taker)?.city;
  const where = city === undefined ? '' : ` in ${ctx.cityName(city)}`;
  const opts = { ...base, players: cap ? [taker, cap.from] : [taker], ...(city === undefined ? {} : { at: ctx.cityPos(city) }) };
  if (!cap) {
    if (taker === ctx.me) return line('victory', `You built the Yam Relay Station${where}: every empire may cross your land at Peace now${gain}`, { ...opts, tone: 'good' });
    return line('victory', `${who(ctx, taker)} built the Yam Relay Station${where}: their land is open to every empire at Peace`, { ...opts, tone: mine(taker) ? 'good' : 'neutral' });
  }
  const from = cap.from;
  if (isBarbarian(from)) return line('victory', `${subject(taker)} took the Yam Relay Station back from the barbarians${where}${gain}`, { ...opts, tone: mine(taker) ? 'good' : 'neutral' });
  if (mine(taker)) return line('victory', `${subject(taker)} took the Yam Relay Station${where} from ${whom(ctx, from)}: the open road is ${taker === ctx.me ? 'yours' : 'your team’s'} now, and every empire crosses ${taker === ctx.me ? 'your' : 'its'} land at Peace${gain}`, { ...opts, tone: 'good' });
  if (mine(from)) return line('victory', `${subject(taker)} took the Yam Relay Station${where} from ${from === ctx.me ? 'you' : whom(ctx, from)}: the open road and the points went with the city. Retake it to get them back${from === ctx.me ? ' (−' + e.vp + ' VP)' : ''}`, { ...opts, tone: 'bad' });
  return line('victory', `${who(ctx, taker)} took the Yam Relay Station${where}: every empire may now cross ${possessiveOf(who(ctx, taker))} land at Peace`, { ...opts, tone: 'neutral' });
}

/** Format one event, with its report icon. UnitMoved steps are collapsed by `formatEvents`. */
export function formatEvent(e: Event, ctx: ReportContext): ReportLine {
  const l = formatEventLine(e, ctx);
  return l.icon ? l : { ...l, icon: eventIcon(e, (p) => ctx.playerFaction?.(p), (p) => ctx.team.includes(p)) };
}

function empireLabel(ctx: ReportContext, team: number): string {
  return ctx.empireName?.(team) ?? ((ctx.teamPlayers?.(team) ?? []).map((id) => {
    const f = ctx.playerFaction?.(id);
    return f ? empireNameOf([{ id, team, faction: f, name: ctx.playerName(id) }]) : ctx.playerName(id);
  }).join(' & ') || `Team ${team}`);
}

/** A team two VP short of its target: the turn report's line for the near-victory alarm (UI only; no event stands behind it). Red for rivals, green for your own team, neutral for allies. */
export function victoryNearLine(near: HoldCard, ctx: ReportContext): ReportLine {
  const own = near.team === ctx.teamId, ally = !own && !!ctx.allied?.(near.team);
  const peoples = (ctx.teamPlayers?.(near.team) ?? []).map((id) => ctx.playerFaction?.(id)).filter((f): f is Faction => !!f).map((f) => `the ${PEOPLE_NAME[f]}`);
  const named = peoples.length ? peoples.join(' and ') : empireLabel(ctx, near.team);
  const subject = own ? ((ctx.teamPlayers?.(near.team).length ?? 1) > 1 ? 'Your team is' : 'You are') : `${named.charAt(0).toUpperCase()}${named.slice(1)} are`;
  return line('victory', `${subject} ${near.target - near.vp} VP from victory: ${near.vp} of ${near.target}`, {
    weight: 2, tone: own ? 'good' : ally ? 'neutral' : 'bad', icon: 'vp', empire: near.team,
    details: [NEAR_RULE],
    reopenHold: { ...near, near: true },
  });
}

/** What stops a team near its target, from rules that exist: some of its VP are held, and held VP moves with the city or wonder that holds it. */
export const NEAR_RULE = 'World wonders and some faction goals score for whoever holds them. Capture the city that holds one to take its VP.';

function formatEventLine(e: Event, ctx: ReportContext): ReportLine {
  // The other side of a diplomacy line that involves the viewer's team: named and marked in its color.
  const other = diplomaticTeams(e).length === 2 && diplomaticTeams(e).includes(ctx.teamId ?? -1)
    ? diplomaticTeams(e).find((t) => t !== ctx.teamId) : undefined;
  const empire = (team: number) => empireLabel(ctx, team);
  const diplomacyLine = (text: string, opts: Partial<ReportLine> = {}) => line('diplomacy', text, {
    players: diplomaticTeams(e).flatMap((t) => ctx.teamPlayers?.(t) ?? []), ...(other === undefined ? {} : { empire: other }), ...opts,
  });
  const mine = (p: PlayerId) => p === ctx.me || ctx.team.includes(p);
  const toneFor = (p: PlayerId, good = true) => (mine(p) ? (good ? 'good' : 'bad') : 'neutral');
  const treaty = (id: number) => ctx.treatyKind?.(id) ?? 'Treaty';
  // Treaties and offers are named by the empires involved, never by their internal ids.
  const between = (a: number, b: number) => a === ctx.teamId ? `with ${empire(b)}` : b === ctx.teamId ? `with ${empire(a)}` : `between ${empire(a)} and ${empire(b)}`;
  const offer = (p: { kind?: Parameters<typeof offerLabel>[0]['kind']; from: number; to: number; required_teams?: number[] }) => {
    if ((p.kind ?? 'Peace') === 'Peace' && ctx.teamId !== undefined && [p.from, p.to].includes(ctx.teamId))
      return `Peace with ${empire(p.from === ctx.teamId ? p.to : p.from)}`;
    return `${offerLabel(p)} offer ${ctx.teamId === p.to ? `from ${empire(p.from)}` : ctx.teamId === p.from ? `to ${empire(p.to)}` : `from ${empire(p.from)} to ${empire(p.to)}`}`;
  };
  const peaceWar = (p: { from: number; to: number }) => ctx.teamId === p.from || ctx.teamId === p.to
    ? `your war with ${empire(ctx.teamId === p.from ? p.to : p.from)}`
    : `the war between ${empire(p.from)} and ${empire(p.to)}`;
  const peaceDetails = (p: { kind?: string; from: number; to: number; required_teams?: number[] }, accepted = false) => {
    if ((p.kind ?? 'Peace') !== 'Peace' || !p.required_teams?.length) return [];
    const continued = (ctx.peaceContinuing?.(p.from, p.to) ?? []).map(([a, b]) => {
      const owner = a === ctx.teamId || b === ctx.teamId ? ctx.teamId : [a, b].includes(p.from) ? p.from : p.to;
      const enemy = a === owner ? b : a;
      return owner === ctx.teamId ? `Your war with ${empire(enemy)} continues.`
        : `The war between ${empire(owner!)} and ${empire(enemy)} continues.`;
    });
    return [[`Peace ${accepted ? 'ended' : 'would end'} only ${peaceWar(p)}.`,
      ...continued, accepted ? 'Both teams agreed.' : 'Both teams must agree.'].join(' ')];
  };
  const rename = (text: string) => {
    const subject = ctx.renameSubject?.(text) ?? text;
    return ctx.renamePlayers?.(subject) ?? subject;
  };
  // Core names a team by its players' names ("Alex & Blake"): find which side that is.
  const teamByLabel = (label: string, teams: number[]) => teams.find((t) =>
    (ctx.teamPlayers?.(t) ?? []).map((id) => ctx.playerName(id)).join(' & ') === label);
  const terms = allianceTerms();
  const sideName = (t: number) => (t === ctx.teamId ? 'you' : empire(t));
  switch (e.type) {
    case 'CityRazed': {
      if (e.ordered) return line('combat', `${who(ctx, e.by)} razed ${at(ctx, e.at)}${e.sack_gold === undefined ? '' : ` and gained ${e.sack_gold} Gold`}`, {
        at: e.at, players: [e.by, e.from], weight: mine(e.by) ? 2 : 1,
        tone: mine(e.by) ? 'neutral' : 'bad',
      });
      const took = [
        ...(e.sack_gold === undefined ? [] : [`carried off ${e.sack_gold} Gold`]),
        ...(e.camp === undefined ? [] : ['raised a warband camp near the ruins']),
      ];
      const list = took.length > 1 ? `${took.slice(0, -1).join(', ')} and ${took[took.length - 1]}` : took[0];
      return line('combat', `${who(ctx, e.by)} sacked ${e.from === ctx.me ? 'your' : possessiveOf(who(ctx, e.from))} city ${at(ctx, e.at)}${list ? ` and ${list}` : ''}`, {
        at: e.at, players: [e.by, e.from], weight: mine(e.by) || mine(e.from) ? 2 : 1,
        tone: mine(e.from) ? 'bad' : mine(e.by) ? 'good' : 'neutral',
        details: [
          ...(e.camp === undefined ? [] : [`The new Celtic warband camp stands ${at(ctx, e.camp)}.`]),
        ],
      });
    }
    case 'TreatyActivated': {
      const r = e.relation;
      const until = r.security?.protected_until;
      const upkeep = terms ? `${terms.upkeep} Gold a turn for every city it owns, and never less than ${terms.upkeep} Gold` : 'Gold for each owned city every turn';
      return diplomacyLine(`Alliance ${between(r.a, r.b)} signed · begins turn ${r.effective_turn}`, { weight: 2,
        details: [ALLIANCE_SHARES, `Each empire pays ${upkeep}.`,
          ...(until ? [`Protected until turn ${until}: leaving before then costs ${terms ? `${terms.breachCost} Gold` : 'Gold'} and is a betrayal.`] : [])] });
    }
    case 'TreatyEnded': {
      // "<team> left the Alliance": named from the viewer's side. An ally leaving you is bad news.
      const left = /^(.*) left the Alliance$/.exec(e.reason);
      const leaver = left ? teamByLabel(left[1], [e.a, e.b]) : undefined;
      if (leaver === undefined) {
        const reason = rename(e.reason);
        return diplomacyLine(`${treaty(e.treaty_id)} ${between(e.a, e.b)} ended turn ${e.effective_turn}${reason === 'Alliance ended' || reason === 'Alliance broken' ? '' : `: ${reason}`}`, { weight: 2 });
      }
      const stayer = leaver === e.a ? e.b : e.a;
      return diplomacyLine(`${cap(sideName(leaver))} left the Alliance with ${sideName(stayer)} · turn ${e.effective_turn}`, {
        weight: 2, tone: stayer === ctx.teamId ? 'bad' : 'neutral',
        details: ['Sight is no longer shared and borders close: both teams are at Peace.'],
      });
    }
    // Broken against you, it is "with you": `between` names the other side, which is the breaker itself.
    case 'TreatyBreached': return diplomacyLine(`${who(ctx, e.player)} broke the Alliance ${e.with === ctx.teamId ? 'with you' : between(e.by, e.with)} early · ${e.cost} Gold · turn ${e.effective_turn}`, {
      weight: 2,
      details: [`Leaving within ${terms.protectedTurns} turns of signing costs the leaver ${e.cost} Gold.`],
      // Broken against you, it is bad news; breaking it yourself cost your Gold.
      tone: e.with === ctx.teamId || mine(e.player) ? 'bad' : 'neutral',
    });
    case 'TreatyPayment': return line('economy', `${e.player === ctx.me ? 'Paid' : `${who(ctx, e.player)} paid`} ${e.gold} Gold: ${e.cause}`, {
      players: [e.player], weight: 1, tone: mine(e.player) ? 'bad' : 'neutral',
    });
    case 'AllianceBetrayed': {
      const b = e.betrayal;
      const them = b.by === ctx.teamId ? 'you' : 'them';
      return diplomacyLine(`${cap(sideName(b.by))} betrayed ${sideName(b.against)}: ${sideName(b.against)} won't ally with ${them} until turn ${b.until}`, {
        weight: 2,
        details: [`Leaving an Alliance within ${terms.protectedTurns} turns of signing, or declaring war on an ally, is a betrayal: the betrayed team won't ally with the betrayer for ${b.until - b.turn} turns.`],
        // Betrayed, you lost an ally; betraying, you're shut out of an Alliance with them. Both cost you.
        tone: b.against === ctx.teamId || b.by === ctx.teamId ? 'bad' : 'neutral',
      });
    }
    case 'WarDeclared': {
      const team = (id: number) => ctx.teamName?.(id) ?? `Team ${id}`;
      const r = e.relation;
      if (ctx.turn !== undefined && r.effective_turn === ctx.turn) {
        // Surprise war (G20): no treaty, so it started this turn, before any combat.
        const victim = ctx.teamId !== undefined && [r.a, r.b].includes(ctx.teamId) && !r.declarers.includes(ctx.teamId);
        if (victim) {
          const by = r.declarers[0];
          const theirs = (owner: PlayerId) => r.declarers.some((t) => (ctx.teamPlayers?.(t) ?? []).includes(owner));
          const hit = (ctx.events ?? []).find((x) => (x.type === 'Battle' && theirs(x.attacker_owner) && x.defender.kind === 'Unit' && mine(x.defender.owner))
            || (x.type === 'SettlerCaptured' && theirs(x.to) && mine(x.from)));
          const what = hit?.type === 'Battle' && hit.defender.kind === 'Unit'
            ? `attacked ${unitLabel(ctx, hit.defender.unit, hit.defender.owner, hit.defender.unit_kind).replace(/^Your /, 'your ')} ${at(ctx, hit.at)}`
            : hit?.type === 'SettlerCaptured' ? `captured your Settler ${at(ctx, hit.at)}` : 'on you: War started this turn';
          return diplomacyLine(`${empire(by)} declared war ${hit ? 'and ' : ''}${what}`, {
            weight: 2, tone: 'bad',
            details: ['No treaty stood between you, so the declaration took effect at once (a surprise war).'],
          });
        }
        return diplomacyLine(`War: ${team(r.a)} and ${team(r.b)} · started this turn`, {
          weight: 2, tone: 'neutral', details: [`Declared by ${r.declarers.map(team).join(' and ')}.`,
            r.declarers.includes(ctx.teamId ?? -1)
              ? 'No treaty stood between you, so the war began at once and the attacks you ordered took place this turn.'
              : 'No treaty stood between them, so War took effect at once and attacks ordered with it resolved this turn.'],
        });
      }
      const onYou = ctx.teamId !== undefined && [r.a, r.b].includes(ctx.teamId) && !r.declarers.includes(ctx.teamId);
      return diplomacyLine(`War: ${team(r.a)} and ${team(r.b)} · begins turn ${r.effective_turn}`, {
        weight: 2, tone: onYou ? 'bad' : 'neutral', details: [`Declared by ${r.declarers.map(team).join(' and ')}.`,
          `Turn ${r.effective_turn - 1} resolved under Peace. Both teams can plan before the first combat in turn ${r.effective_turn}.`],
      });
    }
    case 'WarJoined': {
      // `e.team` went to war with the other side of the relation, for its ally `e.ally`: it defended
      // an ally the enemy attacked (the enemy declared), or answered its ally's call to arms.
      const enemy = e.relation.a === e.team ? e.relation.b : e.relation.a;
      const name = empire;
      const you = ctx.teamId;
      const defended = e.relation.declarers.includes(enemy);
      const alliedNow = (ctx.events ?? []).some((event) => event.type === 'TreatyActivated'
        && event.relation.state === 'Alliance'
        && ((event.relation.a === e.team && event.relation.b === e.ally)
          || (event.relation.b === e.team && event.relation.a === e.ally)));
      if (alliedNow && enemy === you) {
        const joining = name(e.team);
        const subject = /^(Mongols|Aztecs|Celts)$/.test(joining) ? `The ${joining}` : joining;
        return diplomacyLine(`${subject} allied with ${name(e.ally)} and joined its war against you`, {
          weight: 2, tone: 'bad', details: ['Allies defend each other at once.'],
        });
      }
      const text = defended
        ? (e.team === you ? `You came to the defence of ${name(e.ally)} against ${name(enemy)}`
          : enemy === you ? `${name(e.team)} came to the defence of ${name(e.ally)} against you`
          : e.ally === you ? `${name(e.team)} came to your defence against ${name(enemy)}`
          : `${name(e.team)} came to the defence of ${name(e.ally)} against ${name(enemy)}`)
        : (e.team === you ? `You joined ${name(e.ally)}'s war on ${name(enemy)}`
          : enemy === you ? `${name(e.team)} joined ${name(e.ally)}'s war on you`
          : e.ally === you ? `${name(e.team)} joined your war on ${name(enemy)}`
          : `${name(e.team)} joined ${name(e.ally)}'s war on ${name(enemy)}`);
      return diplomacyLine(`${text} · started turn ${e.relation.effective_turn}`, {
        weight: 2, tone: enemy === you ? 'bad' : e.ally === you ? 'good' : 'neutral',
        details: [defended ? 'Allies defend each other at once: an ally that is attacked is joined by its allies the same turn.'
          : 'This ally answered the call to arms and joined the war.'],
      });
    }
    case 'CallToArms': {
      const { ally, target, team } = e.call;
      const text = team === ctx.teamId ? `${empire(ally)} calls you to arms against ${empire(target)}`
        : ally === ctx.teamId ? `You called ${empire(team)} to arms against ${empire(target)}`
        : `${empire(ally)} called ${empire(team)} to arms against ${empire(target)}`;
      return diplomacyLine(`${text} · answer by turn ${e.call.respond_by}`, {
        weight: team === ctx.teamId ? 2 : 1, ...(team === ctx.teamId ? { openInboxTeam: ally } : {}),
        details: ['Joining puts the team at War with the target. Declining keeps the Alliance and the Peace, with no penalty.'],
      });
    }
    case 'CallAnswered': {
      const { ally, target, team } = e.call;
      const who = team === ctx.teamId ? 'You' : empire(team);
      const text = e.joined ? `${who} joined ${ally === ctx.teamId ? 'your' : `${empire(ally)}'s`} war on ${empire(target)}`
        : `${who} declined ${ally === ctx.teamId ? 'your' : `${empire(ally)}'s`} call to arms against ${empire(target)}`;
      return diplomacyLine(`${text} · turn ${e.effective_turn}`, {
        weight: 1, tone: e.joined && ally === ctx.teamId ? 'good' : !e.joined && ally === ctx.teamId ? 'bad' : 'neutral',
        ...(e.joined ? {} : { details: ['The Alliance stands and the Peace with the target stays.'] }),
      });
    }
    case 'CallLapsed': {
      const { ally, target, team } = e.call;
      const text = team === ctx.teamId ? `The call to arms from ${empire(ally)} against ${empire(target)} ended`
        : ally === ctx.teamId ? `Your call to arms to ${empire(team)} against ${empire(target)} ended`
        : `The call to arms from ${empire(ally)} to ${empire(team)} against ${empire(target)} ended`;
      return diplomacyLine(`${text}: ${rename(e.reason)}`, { weight: 1 });
    }
    case 'PeaceEstablished': {
      const r = e.relation;
      const team = (id: number) => ctx.teamName?.(id) ?? `Team ${id}`;
      return diplomacyLine(`Peace: ${team(r.a)} and ${team(r.b)} · begins turn ${r.effective_turn}`, {
        weight: 2, details: [`Guaranteed through turn ${r.guarantee_until - 1}. Earliest new War: turn ${r.guarantee_until}.`,
          'Borders are closed. Units left inside may stay without healing or fortification; each move must approach an exit.'],
      });
    }
    case 'ProposalCreated': return diplomacyLine(`${offer(e.proposal)} ${ctx.teamId === e.proposal.to ? 'arrived' : 'sent'}${e.proposal.kind === 'Peace' && e.proposal.required_teams?.length ? ` · would end ${peaceWar(e.proposal)}` : ''} · respond by turn ${e.proposal.respond_by}`, { weight: 1, details: peaceDetails(e.proposal), ...(ctx.teamId === e.proposal.to ? { openInboxTeam: e.proposal.from } : {}) });
    case 'ProposalAccepted': case 'ProposalRejected': case 'ProposalWithdrawn': case 'ProposalExpired':
      return diplomacyLine(`${offer(e.proposal)} ${e.type.slice(8).toLowerCase()}${e.type === 'ProposalAccepted' && e.proposal.kind === 'Peace' && e.proposal.required_teams?.length ? ` · ended ${peaceWar(e.proposal)}` : ''} · turn ${e.effective_turn}`, { weight: 1, details: e.type === 'ProposalAccepted' ? peaceDetails(e.proposal, true) : [] });
    case 'ProposalInvalidated': return diplomacyLine(`${offer(e.proposal)} invalidated: ${rename(e.reason)}`, { weight: 1 });
    case 'OrderSuperseded': return diplomacyLine(`${possessive(ctx, e.decision.player)} diplomatic order was dropped; ${whom(ctx, e.decision.winner)} gave the order that was used`, {
      weight: 1, players: [e.decision.player, e.decision.winner], details: [
        `The lower-numbered seat's order is used. ${e.decision.winner_order.type} was used; ${e.decision.order.type} was dropped.`,
      ],
    });
    case 'UnitMoved':
      if (!e.from || !e.to) {
        const action = e.to ? `entered sight ${at(ctx, e.to)}` : e.from ? `left sight ${at(ctx, e.from)}` : 'moved beyond sight';
        return line('movement', `${unitLabel(ctx, e.unit, e.owner)} ${action}`, { at: e.to ?? e.from, players: [e.owner] });
      }
      return line('movement', `${unitLabel(ctx, e.unit, e.owner)} moved 1 tile, now ${at(ctx, e.to)}`, {
        at: e.to,
        players: [e.owner],
      });
    case 'MoveBlocked':
      if (e.reason.includes('rode away before your ') && e.reason.endsWith(' reached it.')) {
        return line('orders', rename(e.reason), {
          at: e.at,
          players: [e.owner],
          weight: mine(e.owner) ? 2 : 0,
          tone: toneFor(e.owner, false),
        });
      }
      // Core's stop after repeated blocked turns: the unit needs orders, so it is news, not movement.
      if (e.reason.startsWith(MOVE_STOPPED_PREFIX)) {
        return line('orders', `${unitLabel(ctx, e.unit, e.owner)} stopped and needs orders: ${rename(e.reason.slice(MOVE_STOPPED_PREFIX.length))}`, {
          at: e.at,
          players: [e.owner],
          weight: mine(e.owner) ? 2 : 0,
          tone: toneFor(e.owner, false),
        });
      }
      // A move onto a unit of a team at Peace never attacks it: say so plainly, never as hidden movement.
      if (e.reason.startsWith(PEACE_BLOCK_PREFIX) && mine(e.owner)) {
        const blocker = rename(e.reason.slice(PEACE_BLOCK_PREFIX.length));
        return line('orders', `${unitLabel(ctx, e.unit, e.owner)} couldn't enter ${e.toward ? location(ctx, e.toward) : 'the tile'}: ${blocker} is there and ${e.owner === ctx.me ? "you're" : "they're"} at Peace`, {
          at: e.at, players: [e.owner], weight: 2, tone: 'neutral',
          details: ['A move never attacks a unit of a team at Peace. To attack it, click it and confirm Declare War; or give this unit a new order.'],
        });
      }
      // The attack's target died to ranged fire before movement and nothing hostile is left there.
      if (e.reason === TARGET_GONE) {
        return line('orders', `${unitLabel(ctx, e.unit, e.owner)} didn't attack: its target was destroyed first. Needs orders.`, {
          at: e.at, players: [e.owner], weight: mine(e.owner) ? 2 : 0, tone: 'neutral',
        });
      }
      // A friend held a tile on the route (core `stayed_reason`): name it, from the viewer's side.
      if (e.reason.startsWith(FRIEND_BLOCK_PREFIX) && e.reason.includes(', which ') && mine(e.owner)) {
        return line('orders', `${unitLabel(ctx, e.unit, e.owner)} was blocked ${e.at ? at(ctx, e.at) : 'beyond sight'} by ${rename(e.reason.slice(FRIEND_BLOCK_PREFIX.length))}`, {
          at: e.at, players: [e.owner], weight: 2, tone: toneFor(e.owner, false),
          details: ['Routes go around your units that stay put whenever a free route exists; this turn there was none. Move the blocking unit or give this one another route.'],
        });
      }
      return line('movement', `${unitLabel(ctx, e.unit, e.owner)} was blocked ${e.at ? at(ctx, e.at) : 'beyond sight'}`, {
        at: e.at,
        players: [e.owner],
        details: [rename(e.reason)],
        weight: mine(e.owner) ? 1 : 0,
      });
    case 'AttackPursuit': {
      const prey = unitLabel(ctx, e.target, e.target_owner, e.target_kind);
      const own = unitLabel(ctx, e.unit, e.owner).replace(/^Your /, 'your ');
      const where = e.target_at ? ` ${at(ctx, e.target_at)}` : '';
      const base = { at: e.target_at ?? e.at, players: [e.owner, e.target_owner] };
      switch (e.outcome) {
        case 'Followed': return line('orders', `${prey} slipped away; ${own} followed but couldn't reach it this turn. Needs orders.`, {
          ...base, at: e.at, weight: 2, tone: toneFor(e.owner, false), details: [ONE_TURN_ATTACK],
        });
        case 'OutOfReach': return line('orders', `${cap(own)} couldn't reach ${prey}${where} this turn. Needs orders.`, {
          ...base, at: e.at, weight: 2, details: [ONE_TURN_ATTACK],
        });
        case 'Attacked': return line('combat', `${prey} slipped away; ${own} followed and attacked it${where}`, { ...base, weight: 1 });
        case 'LostSight': return line('orders', `${prey} slipped away; ${own} lost sight of it and needs orders`, {
          ...base, at: e.at, weight: 2, tone: toneFor(e.owner, false),
        });
        case 'AwaitingWar': return line('combat', `${own.replace(/^your /, 'Your ')} will attack ${prey}${where} when War begins next turn`, { ...base, weight: 1 });
      }
      break;
    }
    case 'CityReinforced': {
      // Reinforcing your own city goes first: the unit was the garrison the enemy's attack met.
      const events = ctx.events ?? [];
      const lost = events.some((x) => x.type === 'UnitDestroyed' && x.unit === e.unit);
      const fell = events.some((x) => x.type === 'CityCaptured' && x.city === e.city);
      const city = ctx.cityName(e.city);
      const unit = cap(unitLabel(ctx, e.unit, e.owner, e.unit_kind));
      const text = fell
        ? `${unit} reached ${city} first and fought as its garrison, but the city fell`
        : lost
          ? `${unit} reached ${city} first and fought as its garrison until it was destroyed`
          : `${unit} reached ${city} first and held it`;
      return line('combat', text, {
        at: e.at, players: [e.owner], weight: mine(e.owner) ? 2 : 1,
        tone: mine(e.owner) ? (fell || lost ? 'bad' : 'good') : 'neutral',
        details: ['A unit stepping into its own city arrives before an enemy attack on it and becomes the garrison that attack meets: the city\'s HP absorbs first, then the garrison fights.'],
      });
    }
    case 'Bounce':
      return line('movement', `${e.units.length} units contested ${location(ctx, e.tile)} and all stayed put`, {
        at: e.tile,
        details: ['Friendly units stepping into the same tile bounce; their destinations are kept.'],
      });
    case 'Battle': return battleLine(e, ctx, mine);
    case 'UnitDestroyed': {
      // Name who did it and from where, from this turn's battle, when the viewer saw it.
      const fight = (ctx.events ?? []).find((x): x is Battle => x.type === 'Battle'
        && ((x.defender.kind === 'Unit' && x.defender.unit === e.unit && x.defender_hp_after <= 0)
          || (x.attacker === e.unit && x.attacker_hp_after <= 0)));
      const killer = fight && (fight.attacker === e.unit ? null : fight);
      const by = killer
        ? typeof killer.attacker === 'number'
          ? ` by ${unitLabel(ctx, killer.attacker, killer.attacker_owner).replace(/^Your /, 'your ')}${battleFrom(killer, mine)}`
          : ` by ${killer.attacker_owner === ctx.me ? '' : `${possessiveOf(who(ctx, killer.attacker_owner))} city `}${ctx.cityName(killer.attacker.city)}`
        : '';
      const other = fight ? (fight.attacker === e.unit ? fight.defender.owner : fight.attacker_owner) : null;
      return line('combat', `${unitLabel(ctx, e.unit, e.owner, e.unit_kind)} was destroyed ${at(ctx, e.at)}${by}`, {
        at: e.at,
        from: killer ? (killer.attacker_at ?? null) : null,
        players: [e.owner],
        weight: 1,
        tone: mine(e.owner) ? 'bad' : other !== null && mine(other) ? 'good' : 'neutral',
      });
    }
    case 'SpartaKillHeal':
      return line('combat', `Spartan units healed ${e.hp_healed} HP by destroying ${possessive(ctx, e.victim_owner)} ${itemName({ kind: 'Unit', unit: e.unit_kind })}`, {
        players: [e.player], tone: toneFor(e.player), weight: 2,
      });
    case 'UnitAwakened':
      return line('orders', `${unitLabel(ctx, e.unit, e.owner, e.unit_kind)} woke: enemy military in sight within movement range. Needs orders.`, {
        at: e.at, players: [e.owner], weight: 2, tone: 'neutral',
      });
    case 'UnitHealed':
      return line('orders', `${unitLabel(ctx, e.unit, e.owner, e.unit_kind)} healed to full. Needs orders.`, {
        at: e.at, players: [e.owner], weight: 1, tone: 'good', icon: 'mark-heal',
      });
    case 'AutomationStopped':
      return line('orders', `${unitLabel(ctx, e.unit, e.owner)} stopped auto-explore: ${STOP_REASON[e.reason]}. Needs orders.`, {
        at: e.at, players: [e.owner], weight: 2, tone: 'neutral',
      });
    case 'UnitDisbanded':
      // The reason is the news: never hide it behind the expander (F10).
      return line('orders', `${unitLabel(ctx, e.unit, e.owner)} was disbanded: ${e.reason}`, {
        players: [e.owner],
        tone: toneFor(e.owner, false),
      });
    case 'SettlerCaptured':
      if (isBarbarian(e.from)) {
        return line('combat', `${who(ctx, e.to)} freed a captive Settler from the barbarians ${at(ctx, e.at)}`, {
          at: e.at, players: [e.to], weight: 2, tone: toneFor(e.to),
        });
      }
      return line('combat', `${who(ctx, e.to)} captured ${e.from === ctx.me ? 'your' : possessiveOf(who(ctx, e.from))} Settler ${at(ctx, e.at)}`, {
        at: e.at,
        players: [e.from, e.to],
        weight: 2,
        tone: mine(e.to) ? 'good' : mine(e.from) ? 'bad' : 'neutral',
      });
    case 'CityFounded':
      return line('cities', `${who(ctx, e.owner)} founded ${e.name}`, {
        at: e.at,
        players: [e.owner],
        weight: mine(e.owner) ? 2 : 1,
        tone: toneFor(e.owner),
      });
    case 'FlowerWars':
      return line('cities', `Flower Wars: ${ctx.cityName(e.city)} gains 1 population (now ${e.population})`, { at: ctx.cityPos(e.city), players: [e.owner], weight: 1, tone: toneFor(e.owner) });
    case 'CityGrew': {
      // Name what the new population unlocks (e.g. the Settler at 2), so growth is never a surprise.
      const unlocked = mine(e.owner) ? ctx.growthUnlocks?.(e.city, e.population) : null;
      return line('cities', `${ctx.cityName(e.city)} grew to ${e.population} population${unlocked ? ` · unlocks ${unlocked}` : ''}`, {
        at: ctx.cityPos(e.city),
        players: [e.owner],
        weight: unlocked ? 2 : 0,
        tone: toneFor(e.owner),
      });
    }
    case 'CityStarved':
      return line('cities', `${ctx.cityName(e.city)} is starving: population fell to ${e.population}`, {
        at: ctx.cityPos(e.city),
        players: [e.owner],
        weight: 1,
        tone: toneFor(e.owner, false),
      });
    case 'CityCaptured': {
      // A combined assault that broke the city and its garrison in one stage is one line.
      const assault = combinedAssaults(ctx.events ?? []).get(assaultKey({ kind: 'City', city: e.city, owner: e.from }));
      const garrisonLost = (ctx.events ?? []).some((x) => x.type === 'UnitDestroyed' && x.owner === e.from
        && hexEq(x.at, ctx.cityPos(e.city)));
      const bounty = e.bounty ? ` and seized ${e.bounty} Gold` : '';
      const text = assault
        ? `${ctx.cityName(e.city)} fell to a combined assault by ${assaultForce(ctx, assault)}; ${garrisonLost ? 'its garrison was destroyed and the city captured' : 'the city was captured'}${bounty}. Its counterattack was split, ${counterSplit(assault)}`
        : `${who(ctx, e.to)} captured ${ctx.cityName(e.city)} from ${whom(ctx, e.from)}${bounty}`;
      return line('combat', text, {
        at: ctx.cityPos(e.city),
        players: [e.from, e.to],
        weight: 2,
        tone: mine(e.to) ? 'good' : mine(e.from) ? 'bad' : 'neutral',
        details: [
          ...(assault ?? []).map((b) => `${unitLabel(ctx, b.attacker as number, b.attacker_owner)} dealt ${b.damage_to_defender} and took ${b.damage_to_attacker}.`),
          `Captured cities lose half their population (never below 1) and resist for ${CAPTURE_RESISTANCE_TURNS} turns (no output).`,
        ],
      });
    }
    case 'AthensCityCaptured':
      return line('science', mine(e.to)
        ? `Captured ${ctx.cityName(e.city)} from Athens: you gain one free technology. Choose it.`
        : `${who(ctx, e.to)} captured ${ctx.cityName(e.city)} from Athens: one free technology`, {
        at: ctx.cityPos(e.city), players: [e.from, e.to], weight: 2,
        tone: mine(e.to) ? 'good' : mine(e.from) ? 'bad' : 'neutral',
        details: ['The captor chooses a technology from its current era. Each empire can earn this reward once per city.'],
      });
    case 'SpartaConquest':
      return line('combat', `Spartan units healed by the conquest of ${ctx.cityName(e.city)}`, {
        at: ctx.cityPos(e.city), players: [e.player], weight: 2, tone: toneFor(e.player),
        details: [`${e.units_healed} wounded units healed within two tiles.`],
      });
    case 'BuildingDismantled':
      return line('cities', `The ${itemName({ kind: 'Building', building: e.building }, ctx.playerFaction?.(e.owner))} in ${ctx.cityName(e.city)} was dismantled: ${e.reason}`, {
        at: ctx.cityPos(e.city),
        players: [e.owner],
        weight: 1,
        tone: toneFor(e.owner, false),
      });
    case 'HeirBorn': {
      const vp = factionGoalVp('Byzantium');
      if (mine(e.owner)) {
        return line('victory', `${e.owner === ctx.me ? 'Your' : `${possessiveOf(who(ctx, e.owner))}`} Heir was born in ${e.city_name}: hold him to score ${vp} VP, and keep him safe`, {
          at: e.at, players: [e.owner], weight: 2, tone: 'good', icon: 'unit-heir',
          details: ['Every empire was told where he was born. His position is public from now on: a rival that captures him scores the points.'],
        });
      }
      return line('victory', `${possessiveOf(who(ctx, e.owner))} Heir was born in ${e.city_name}: be at War, then capture him to score ${vp} VP`, {
        at: e.at, players: [e.owner], weight: 2, tone: 'neutral', icon: 'unit-heir',
        details: ['Every empire is told where he was born. His position is public every turn, on the map and the minimap, and whoever holds him scores the points. To capture him you must be at War with his holder.'],
      });
    }
    case 'CompanyHired': {
      const unit = withArticle(unitName(ctx, 0, e.seller, e.unit));
      const turns = RULES.company.turns;
      const base = { at: ctx.cityPos(e.city), players: [e.owner, e.seller], weight: 1 as const };
      if (mine(e.owner)) {
        return line('economy', `${e.owner === ctx.me ? 'Hired' : `${who(ctx, e.owner)} hired`} ${unit} Company from ${whom(ctx, e.seller)} for ${e.gold} Gold, for ${turns} turns`, {
          ...base, tone: 'neutral',
          details: [`The Gold went to ${whom(ctx, e.seller)} in full. The Company serves until the end of turn ${e.until}, then goes home, and costs ${RULES.company.upkeep} Gold a turn while it stays.`],
        });
      }
      return line('economy', `+${e.gold} Gold: ${who(ctx, e.owner)} hired ${unit} Company from you`, { ...base, tone: 'good' });
    }
    case 'CompanyDeparted': {
      const name = unitName(ctx, 0, e.owner, e.unit);
      const adjective = e.origin ? FACTION_ADJECTIVE[e.origin] : null;
      const unit = adjective && !name.startsWith(`${adjective} `) ? `${adjective} ${name}` : name;
      return line('economy', `The ${unit} Company's contract ended${mine(e.owner) ? `: it went home ${at(ctx, e.pos)}` : ''}`, {
        at: e.pos, players: [e.owner], weight: 1, tone: 'neutral',
      });
    }
    case 'CompanyTurned': {
      const name = unitName(ctx, 0, e.owner, e.unit);
      const adjective = e.origin ? FACTION_ADJECTIVE[e.origin] : null;
      const unit = adjective && !name.startsWith(`${adjective} `) ? `${adjective} ${name}` : name;
      return line('combat', `Unpaid, ${withArticle(unit)} Company turned on ${whom(ctx, e.owner)} ${at(ctx, e.pos)}`, {
        at: e.pos, players: [e.owner], weight: 2, tone: toneFor(e.owner, false),
        details: [`A Mercenary Company costs ${RULES.company.upkeep} Gold a turn. When the treasury cannot pay, a Company is the first thing lost, and it turns barbarian where it stands.`],
      });
    }
    case 'ItemBought':
      return line('economy', `${ctx.cityName(e.city)} bought ${builtName(e.item, ctx, e.city, ctx.playerFaction?.(e.owner))} for ${e.gold} Gold`, {
        at: ctx.cityPos(e.city),
        players: [e.owner],
      });
    case 'ProductionCompleted':
      return line('cities', `${ctx.cityName(e.city)} completed ${builtName(e.item, ctx, e.city, ctx.playerFaction?.(e.owner))}`, {
        at: ctx.cityPos(e.city),
        players: [e.owner],
        // An aircraft never stands on the map: the report says what it does (the Air wing in the city holds it).
        ...(e.item.kind === 'Unit' && mine(e.owner) && isAircraft(e.item.unit) ? { details: [readyNotice(e.item.unit, ctx.cityName(e.city)).effect] } : {}),
        weight: mine(e.owner) ? 1 : 0,
        tone: toneFor(e.owner),
      });
    case 'MegaprojectStarted':
      return line('victory', `${who(ctx, e.owner)} began the ${itemName({ kind: 'Building', building: e.building }, ctx.playerFaction?.(e.owner))} in ${ctx.cityName(e.city)}`, {
        at: ctx.cityPos(e.city),
        players: [e.owner],
        weight: 2,
        details: ['Only one empire can complete this world wonder. Its holder scores +2 VP.'],
      });
    case 'WonderRaceLost':
      // One wording with the city's decision and the moment card (`wonderRace.ts`).
      return line('cities', wonderLostLine(who(ctx, e.winner), e.building, ctx.cityName(e.city), e.production, e.gold, e.active ?? true), {
        players: [e.owner], at: ctx.cityPos(e.city), weight: 2, tone: mine(e.owner) ? 'bad' : 'good',
      });
    case 'TechResearched':
      // Your own tech names what it unlocked (as growth does) and reopens its unlock card from the report.
      return line('science', `${who(ctx, e.player)} researched ${TECH_NAME[e.tech]}${e.player === ctx.me && ctx.techUnlocks?.(e.tech).length ? ` · unlocks ${ctx.techUnlocks(e.tech).join(', ')}` : ''}`, {
        players: [e.player],
        ...(e.player === ctx.me ? { reopenTech: e.tech } : {}),
        weight: mine(e.player) ? 1 : 0,
        tone: toneFor(e.player),
      });
    case 'ResearchQueueAdvanced':
      return line('science', e.next ? (e.player === ctx.me ? `Next research: ${TECH_NAME[e.next]}` : `${who(ctx, e.player)}: next research is ${TECH_NAME[e.next]}`) : `${mine(e.player) ? '' : `${who(ctx, e.player)}: `}Research queue finished: choose next`, {
        players: [e.player],
        weight: mine(e.player) && !e.next ? 2 : 0,
      });
    case 'AircraftRebased':
      return line('orders', `${mine(e.owner) ? 'Your' : `${possessive(ctx, e.owner)}`} ${UNIT_NAME[e.kind]} rebased from ${ctx.cityName(e.from)} to ${ctx.cityName(e.to)}`, { players: [e.owner] });
    case 'TileBought':
      return line('economy', `${mine(e.owner) ? ctx.cityName(e.city) : `${possessive(ctx, e.owner)} city ${ctx.cityName(e.city)}`} bought a tile for ${e.gold} Gold`, {
        at: e.tile,
        players: [e.owner],
      });
    case 'TileClaimed': {
      const tile = ctx.claimedTile?.(e.tile, e.owner);
      return line('cities', tile ? `${ctx.cityName(e.city)} claimed ${/^[AEIOU]/.test(tile.terrain) ? 'an' : 'a'} ${tile.terrain} tile${tile.yields ? ` (${tile.yields})` : ''}` : `${ctx.cityName(e.city)} claimed a tile ${at(ctx, e.tile)}`, {
        at: e.tile,
        players: [e.owner],
        tone: toneFor(e.owner),
      });
    }
    case 'TileReleased':
      return line('cities', `${ctx.cityName(e.city)} gave up a tile${e.tile ? ` ${at(ctx, e.tile)}` : ''} as its population fell`, {
        at: e.tile,
        players: [e.owner],
      });
    case 'TilesReassigned':
      return line('cities', `${ctx.cityName(e.to)} took over ${e.tiles.length} ${e.tiles.length === 1 ? 'tile' : 'tiles'} from ${ctx.cityName(e.from)} so it can work ${e.tiles.length === 1 ? 'it' : 'them'}`, {
        at: e.tiles[0] ?? null,
        players: [e.owner],
        tone: toneFor(e.owner),
      });
    case 'RuinExplored': {
      const reward = ruinText(e.reward, ctx.cityName, ctx.playerFaction?.(e.owner));
      return line('discoveries', `${unitLabel(ctx, e.unit, e.owner)} explored an ancient ruin: ${reward.effect}`, {
        details: reward.details,
        at: e.at,
        players: [e.owner],
        weight: reward.tier === 'major' ? 2 : 1,
        tone: toneFor(e.owner),
      });
    }
    case 'WonderDiscovered':
      return line('discoveries', `${who(ctx, e.owner)} discovered ${WONDER_INFO[e.wonder].name}: +${e.gold || e.science} ${e.gold ? 'Gold' : 'Science'}`, {
        at: e.at, players: [e.owner], weight: 2, tone: toneFor(e.owner),
        details: [wonderBonusSentence(e.wonder)],
      });
    case 'CapitalTaken':
      return line('victory', `${who(ctx, e.captor)} took the original capital ${e.name}: +1 VP while holding it`, {
        at: ctx.cityPos(e.capital), players: [e.captor], weight: 2, tone: toneFor(e.captor),
      });
    case 'CapitalSacked':
      return line('victory', `${who(ctx, e.captor)} sacked the original capital ${e.name}: +1 VP, kept for good`, {
        players: [e.captor], weight: 2, tone: e.captor === ctx.me ? 'good' : 'neutral',
      });
    case 'GoldGiven':
      return line('economy', e.from === ctx.me
        ? `You gave ${withThe(who(ctx, e.to))} ${e.amount} Gold`
        : e.to === ctx.me ? `${cap(withThe(who(ctx, e.from)))} gave you ${e.amount} Gold` : `${cap(withThe(who(ctx, e.from)))} gave ${withThe(who(ctx, e.to))} ${e.amount} Gold`, {
        players: [e.from, e.to], weight: 2, tone: e.to === ctx.me ? 'good' : 'neutral',
      });
    case 'CapitalLost':
      return line('victory', `${who(ctx, e.player)} no longer ${e.player === ctx.me ? 'hold' : 'holds'} the original capital ${e.name}: −1 VP`, {
        at: ctx.cityPos(e.capital), players: [e.player], weight: 2, tone: e.player === ctx.me ? 'bad' : 'neutral',
      });
    case 'EraEntered': {
      // Public, 1-based era (2 Classical .. 8 Future), sent the turn the previous era's last technology landed: every
      // player sees it, the first empire into an era is a notable line for all; the Age it begins has its own line (`AgeBegan`).
      const named = ctx.eraName?.(e.era);
      const era = named ?? eraName(e.era - 1);
      const text = `${e.player === ctx.me ? 'You' : who(ctx, e.player)} entered the ${era} era${e.first ? ' first' : ''}`;
      return line(named ? 'victory' : 'science', text, {
        players: [e.player],
        weight: e.first || mine(e.player) ? 2 : 1,
        tone: toneFor(e.player),
        details: e.first ? [`${e.player === ctx.me ? 'You are' : `${who(ctx, e.player)} is`} the first empire into the ${era} era.`] : [],
        ...(e.player === ctx.me ? { reopenEra: { era: e.era, first: e.first } } : {}),
      });
    }
    case 'AgeBegan': {
      // World Ages: the engine's one announcement of an Age. It names no player, so every report filter shows it. It stands in for the first empire's `EraEntered` line (`formatEvents`).
      const era = ctx.eraName?.(e.era) ?? eraName(e.era - 1);
      const mineFirst = e.first === ctx.me;
      return line('victory', `The ${era} Age began: ${mineFirst ? 'You' : who(ctx, e.first)} entered the ${era} era first. Its card is open to every empire.`, {
        players: [], weight: 2, tone: toneFor(e.first), openLedger: true,
        // Every player can see the Age's score check again (the opener its own era-up first).
        reopenEra: { era: e.era, first: mineFirst, age: e.first },
      });
    }
    case 'EraObjectiveScored': {
      const name = ctx.eraName?.(e.era);
      const card = ctx.ageCard?.(e.era, e.slot);
      return line('victory', `${who(ctx, e.player)} scored ${name ? `the ${name} Age` : `Age ${e.era}`} with ${card ? cardName(e.objective) : e.objective}: +1 VP`, {
        players: [e.player], weight: 2, tone: toneFor(e.player),
      });
    }
    case 'GreatWorkCompleted':
      return line('victory', `${who(ctx, e.player)} ${e.again ? 'retook' : 'holds'} the ${BUILDING_NAME[e.building]} (+${e.vp} VP)`, {
        players: [e.player], weight: 2, tone: toneFor(e.player),
      });
    case 'GreatWorkLost':
      return line('victory', `${who(ctx, e.player)} lost the ${BUILDING_NAME[e.building]} (−${e.vp} VP)`, {
        players: [e.player], weight: 2, tone: mine(e.player) ? 'bad' : 'good',
      });
    case 'FactionGoalCompleted':
      if (e.faction === 'Byzantium') return heirGoalLine(e, ctx);
      if (e.faction === 'Mongols') return hallGoalLine(e, ctx);
      if (e.faction === 'India') {
        const captured = ctx.events?.find((x): x is Extract<Event, { type: 'CityCaptured' }> => x.type === 'CityCaptured' && x.to === e.player);
        const built = ctx.events?.find((x): x is Extract<Event, { type: 'ProductionCompleted' }> => x.type === 'ProductionCompleted' && x.owner === e.player && x.item.kind === 'Building' && x.item.building === 'ImperialDiwan');
        const city = captured?.city ?? built?.city;
        const place = city === undefined ? '' : ` in ${ctx.cityName(city)}`;
        return line('victory', `${who(ctx, e.player)} ${captured ? 'took' : 'built'} the Imperial Diwan${place} (+${e.vp} VP while holding its city)`, {
          players: [e.player], weight: 2, tone: toneFor(e.player), icon: 'building-imperialdiwan',
        });
      }
      return line('victory', `${who(ctx, e.player)} ${e.again ? 'regained' : 'completed'} ${FACTION_GOAL[e.faction].name} (+${e.vp} VP)`, {
        players: [e.player],
        weight: 2,
        tone: toneFor(e.player),
      });
    case 'FactionGoalLost':
      if (e.faction === 'Byzantium') return heirGoalLine(e, ctx);
      if (e.faction === 'Mongols') return hallGoalLine(e, ctx);
      if (e.faction === 'India') return line('victory', `${who(ctx, e.player)} lost the Imperial Diwan (−${e.vp} VP): retake its city to score again`, {
        players: [e.player], weight: 2, tone: mine(e.player) ? 'bad' : 'good', icon: 'building-imperialdiwan',
      });
      if (e.faction === 'Celts') {
        return line('victory', `${who(ctx, e.player)} lost ${FACTION_GOAL.Celts.name} (−${e.vp} VP): a city has no wild camp left${mine(e.player) ? '. Raise the Warbands again to raise one in every city' : ''}`, {
          players: [e.player],
          weight: 2,
          tone: mine(e.player) ? 'bad' : 'good',
        });
      }
      // The treasury goal names its amount: "until you hold 5,000 Gold again".
      const hoard = e.faction === 'Switzerland' ? factionGoalRule('Switzerland').match(/Keep ([\d,]+ Gold)/)?.[1] : undefined;
      return line('victory', `${who(ctx, e.player)} lost ${FACTION_GOAL[e.faction].name} (-${e.vp} VP) until ${hoard ? `${e.player === ctx.me ? 'you' : 'they'} hold ${hoard} again` : 'its condition holds again'}`, {
        players: [e.player],
        weight: 2,
        // A rival losing its goal is good for you; your own loss is bad.
        tone: mine(e.player) ? 'bad' : 'good',
      });
    case 'SwarmUnleashed': {
      const lost = e.cities.reduce((n, c) => n + c.lost, 0), tiles = e.cities.reduce((n, c) => n + c.tiles, 0);
      const cities = e.cities.map((c) => `${ctx.cityName(c.city)} −${c.lost}`).join(', ');
      return line('victory', `${who(ctx, e.player)} kindled the New Fire: ${lost} population sacrificed across ${e.cities.length} ${e.cities.length === 1 ? 'city' : 'cities'}, ${tiles} ${tiles === 1 ? 'tile' : 'tiles'} released${mine(e.player) ? ' (+2 VP, kept for good)' : ': its cities are thin'}`, {
        players: [e.player],
        weight: 2,
        tone: mine(e.player) ? 'good' : 'neutral',
        details: [cities],
      });
    }
    case 'SakokuClosed': {
      const turns = sakokuTarget();
      return line('victory', mine(e.player)
        ? `You closed the country (Sakoku): no war, no Alliances, no tech diffusion, and nobody may cross your land at Peace. After ${turns} turns closed, you hold +2 VP`
        : `${who(ctx, e.player)} closed the country (Sakoku): it can't declare war or ally, learns nothing from abroad, and nobody may cross its land at Peace. After ${turns} turns closed it holds 2 VP`, {
        players: [e.player],
        weight: 2,
        tone: 'neutral',
      });
    }
    case 'SakokuReopened':
      return line('victory', mine(e.player)
        ? `You reopened the country after ${e.turns} ${e.turns === 1 ? 'turn' : 'turns'} closed: Sakoku and its VP are lost, and the count restarts`
        : `${who(ctx, e.player)} reopened the country after ${e.turns} ${e.turns === 1 ? 'turn' : 'turns'} closed: its Sakoku VP is gone, and it can declare war and ally again`, {
        players: [e.player],
        weight: 2,
        tone: mine(e.player) ? 'neutral' : 'good',
      });
    case 'SignalBroadcast': {
      const names = e.receivers.map((r) => who(ctx, r));
      const received = e.receivers.includes(ctx.me);
      const text = e.from === ctx.me
        ? `You shared learning: ${names.length ? `${names.join(', ')} each gain` : 'nobody gains'} ${e.techs_each} free technologies of their choice (+2 VP, kept for good)`
        : received
          ? `${who(ctx, e.from)} shared learning: you gain ${e.techs_each} free technologies of your choice. Choose them.`
          : `${who(ctx, e.from)} shared learning: ${names.length ? `${names.join(', ')} each gain` : 'nobody gains'} ${e.techs_each} free technologies`;
      return line('victory', text, {
        players: [e.from, ...e.receivers],
        weight: 2,
        tone: mine(e.from) || received ? 'good' : 'neutral',
      });
    }
    case 'RoadsLaid': {
      const city = ctx.cityName(e.city);
      return line('victory', mine(e.owner)
        ? `The Yam Network: roads now link every city to ${city}`
        : `${who(ctx, e.owner)} built the Yam Relay Station: roads now link every city to ${city}. Capture it for its 2 VP.`, {
        players: [e.owner],
        at: e.at,
        weight: 2,
        tone: mine(e.owner) ? 'good' : 'neutral',
        details: [`${e.linked} ${e.linked === 1 ? 'city' : 'cities'} joined. A road tile costs 1 move; a unit that starts its turn on a road has +1 move (+2 with Railways). Roads never close a border.`],
      });
    }
    case 'VeteranRetired': {
      const last = e.retired >= e.of, name = unitName(ctx, e.unit, e.player, e.kind);
      return line('victory', `${who(ctx, e.player)} retired a ${name} with honour in ${ctx.cityName(e.city)}: ${e.retired} of ${e.of} toward ${FACTION_GOAL.Sparta.name}${last ? ' (+2 VP, kept for good)' : ''}`, {
        players: [e.player],
        weight: last ? 2 : 1,
        tone: 'good',
      });
    }
    case 'WildCalled':
      return line('victory', e.player === ctx.me || ctx.team.includes(e.player)
        ? `${e.player === ctx.me ? 'You' : who(ctx, e.player)} called the warbands: ${e.camps} wild ${e.camps === 1 ? 'camp now stands' : 'camps now stand'} in ${e.camps === 1 ? 'your territory' : 'your cities'}`
        : `${who(ctx, e.player)} called the warbands: ${e.camps} wild ${e.camps === 1 ? 'camp now stands' : 'camps now stand'} in its land. Clearing one takes its VP away, and pays the usual Gold`, {
        players: [e.player],
        weight: 2,
        tone: e.player === ctx.me || ctx.team.includes(e.player) ? 'good' : 'neutral',
      });
    case 'FreeTechsChosen':
      return line('science', `${who(ctx, e.player)} chose ${e.techs.length} free ${e.techs.length === 1 ? 'technology' : 'technologies'}: ${e.techs.map((t) => TECH_NAME[t]).join(', ')}${e.from !== null ? ` (from ${possessiveOf(who(ctx, e.from))} broadcast)` : ''}`, {
        players: [e.player],
        weight: 1,
        tone: toneFor(e.player),
      });
    case 'UnitPromoted':
      return line('combat', `${unitLabel(ctx, e.unit, e.owner, e.unit_kind)} was promoted to level ${e.level} (+${e.level * STRENGTH_PERCENT}% strength)`, {
        at: e.at,
        players: [e.owner],
        weight: 1,
        tone: toneFor(e.owner),
        promotion: { level: e.level, faction: ctx.playerFaction?.(e.owner) },
      });
    case 'ResourcesRevealed': {
      const name = RESOURCE_INFO[e.resource].name;
      const tech = RESOURCE_INFO[e.resource].revealedBy;
      return e.city === null
        ? line('discoveries', `${tech ? TECH_NAME[tech] : 'Research'} reveals ${name}: you have no ${name} deposit inside your borders yet. ${fixClause(e.resource, false, ctx.playerFaction?.(e.player))}`, {
          players: [e.player],
          weight: mine(e.player) ? 1 : 0,
          tone: toneFor(e.player),
        })
        : line('economy', `${name} found near ${ctx.cityName(e.city)}: +${e.supply} ${name} supply`, {
          players: [e.player],
          weight: mine(e.player) ? 2 : 0,
          tone: toneFor(e.player),
        });
    }
    case 'SupplyShortage': {
      const name = RESOURCE_INFO[e.resource].name;
      const idleNames = mine(e.player) ? ctx.idleSupplyNames?.(e.resource) ?? [] : [];
      const users = idleNames.length > 0
        ? ` Idle: ${idleNames.slice(0, 3).join(', ')}${idleNames.length > 3 ? `, and ${idleNames.length - 3} more` : ''}.`
        : '';
      return e.idle === 0
        ? line('economy', `${name} supply restored: every building and unit works again`, { players: [e.player], weight: mine(e.player) ? 2 : 0, tone: toneFor(e.player) })
        : line('economy', `Short of ${name}: ${idleCountText(e.idle)}.${users} You have ${e.supplied > 0 ? `${e.supplied} ${name}` : `no ${name}`} and need ${e.used}. ${fixClause(e.resource, e.supplied > 0, ctx.playerFaction?.(e.player))}`, {
          at: ctx.idleSupplyAt?.(e.resource) ?? null, players: [e.player], weight: mine(e.player) ? 2 : 0, tone: toneFor(e.player, false),
        });
    }
    case 'SiegeBought':
      return line('economy', `${unitLabel(ctx, e.unit, e.owner)} bought Siege for ${e.gold} Gold: its attacks on a city ignore enemy Walls`, {
        players: [e.owner],
      });
    case 'UnitUpgraded':
      return line('economy', `${unitLabel(ctx, e.unit, e.owner)} upgraded to ${itemName({ kind: 'Unit', unit: e.to }, ctx.playerFaction?.(e.owner))} for ${e.gold} Gold`, {
        players: [e.owner],
      });
    case 'OrderFailed': {
      const production = cityProductionFailure(e.reason);
      if (production) {
        const at = ctx.cityNamed?.(production.city) ?? null;
        return line('cities', e.player === ctx.me ? production.text : `${who(ctx, e.player)}: ${production.text}`, {
          at, players: [e.player], weight: mine(e.player) ? 1 : 0, tone: toneFor(e.player, false),
        });
      }
      // Core names units by id ("Settler 62 could not found a city"): name them by kind and place.
      const unit = (id: number) => {
        const pos = ctx.unitPos?.(id);
        return `${unitLabel(ctx, id, e.player)}${pos ? ` ${at(ctx, pos)}` : ''}`;
      };
      const named = (text: string) => rename(text.replace(/\b(?:Settler|[Uu]nit) (\d+)\b/g, (_, id: string) => unit(Number(id)).replace(/^Your /, 'your ')));
      // A reason that starts with the unit is the whole sentence: "Your Settler at Brasslock could not found a city: …".
      const lead = /^(?:Settler|Unit) (\d+)(:?) (.*)$/s.exec(e.reason);
      const text = lead ? `${unit(Number(lead[1]))}${lead[2]} ${named(lead[3])}`
        : e.player === ctx.me ? `Order failed: ${named(e.reason)}` : `${who(ctx, e.player)}: order failed: ${named(e.reason)}`;
      return line('orders', text, {
        players: [e.player],
        weight: mine(e.player) ? 1 : 0,
        tone: toneFor(e.player, false),
      });
    }
    case 'PlayerEliminated':
      return line('victory', e.player === ctx.me ? 'You have been eliminated' : `${who(ctx, e.player)} has been eliminated`, {
        players: [e.player],
        weight: 2,
        tone: toneFor(e.player, false),
      });
    case 'VictoryHoldStarted': {
      // Public to everyone, so no player filter hides it: good for the team itself, bad for its rivals, neutral for its allies.
      const own = e.team === ctx.teamId, ally = !own && !!ctx.allied?.(e.team);
      const subject = own ? ((ctx.teamPlayers?.(e.team).length ?? 1) > 1 ? 'Your team' : 'You') : empire(e.team);
      return line('victory', `${subject} reached ${e.target} VP and must hold it for ${turnsWord(e.turns_needed)} to win`, {
        weight: 2, tone: own ? 'good' : ally ? 'neutral' : 'bad',
        details: [holdRule(e.target, e.turns_needed), 'The turn it was reached is the first turn held.'],
        reopenHold: { team: e.team, vp: e.vp, target: e.target, turnsNeeded: e.turns_needed },
      });
    }
    case 'VictoryHoldBroken': {
      // Bad for the team and its allies, good for its rivals.
      const own = e.team === ctx.teamId, ally = !own && !!ctx.allied?.(e.team);
      return line('victory', `${own ? 'Your' : possessiveOf(empire(e.team))} hold on ${e.target} VP was broken after ${e.turns_held} of ${HOLD_TURNS} turns`, {
        weight: 2, tone: own || ally ? 'bad' : 'good',
        details: [`VP fell to ${e.vp}, below the ${e.target} needed, so the count is back to zero.`],
      });
    }
    case 'GameWon':
      return line('victory', `${e.team === ctx.teamId ? ((ctx.teamPlayers?.(e.team).length ?? 1) > 1 ? 'Your team wins' : 'You win') : `${empire(e.team)} wins`}: ${e.reason}`, {
        weight: 2, tone: ctx.teamId === undefined ? 'neutral' : e.team === ctx.teamId ? 'good' : 'bad',
      });
    case 'GameDrawn':
      return line('victory', `The game ends in a draw: ${e.reason}`, { weight: 2, tone: 'neutral' });
    case 'BarbarianCampSpawned':
      return line('combat', `A barbarian camp appeared ${at(ctx, e.at)}`, { at: e.at });
    case 'BarbarianCampDiscovered':
      return line('discoveries', `Barbarian camp sighted ${at(ctx, e.at)}`, {
        at: e.at,
        players: ctx.team,
        weight: 1,
        details: [`Destroy its guard, then move a military unit onto the camp for ${CAMP_CLEAR_GOLD} Gold. ${ctx.playerFaction?.(ctx.me) === 'Celts' ? 'Barbarians never attack you.' : 'Its scouts bring raiders to cities they find.'}`],
      });
    case 'BarbarianScoutSpotted':
      return line('combat', `A barbarian scout spotted ${ctx.cityName(e.city)} and is heading home. Kill it before it reaches its camp, or raiders will follow.`, {
        at: e.at,
        players: [e.owner],
        weight: 2,
        tone: toneFor(e.owner, false),
      });
    case 'BarbarianRaidLaunched':
      return line('combat', `Barbarian raiders are gathering to raid ${ctx.cityName(e.city)} from a camp to the ${e.direction}`, {
        at: ctx.cityPos(e.city),
        players: [e.owner],
        weight: 2,
        tone: toneFor(e.owner, false),
        details: ['Three raiders muster at the camp, one per turn, then prowl around the city picking off units in the open and capturing civilians. They never assault or take the city. Clear the camp to stop future raids.'],
      });
    case 'BarbarianCampCleared':
      if (e.unit === null) {
        return line('combat', `${possessive(ctx, e.by)} borders dispersed a barbarian camp ${at(ctx, e.at)}`, {
          at: e.at, players: [e.by], weight: 1, tone: toneFor(e.by),
        });
      }
      const gains = `+${e.gold} Gold` + (e.gold === 0 ? ' (no Gold for a camp your own rite raised)' : '');
      return line('combat', `${unitLabel(ctx, e.unit, e.by)} cleared a barbarian camp ${at(ctx, e.at)}: ${gains}`, {
        at: e.at,
        players: [e.by],
        weight: 2,
        tone: toneFor(e.by),
      });
  }
}

/**
 * Format a whole turn. Consecutive single-step UnitMoved events of one unit collapse into one
 * line ("moved 3 hexes to x,y").
 */
/**
 * A core order refusal in the words a player reads: "Astra can't train a Spearman: your levies are full (3 of 2)."
 * Other reasons keep core's wording after the contraction.
 */
export function plainOrderError(message: string): string {
  const m = /^(?:Queue item \d+: )?(.+?) cannot produce (.+?): (.+)$/s.exec(message);
  if (!m) return message;
  const [, city, item, why] = m;
  const levy = /^Levies full: (\d+) of (\d+)$/.exec(why);
  const article = /^[AEIOU]/.test(item) ? 'an' : 'a';
  if (levy) return `${city} can't train ${article} ${item}: your levies are full (${levy[1]} of ${levy[2]}).`;
  return `${city} can't build ${item}: ${plainBlocker(why)}.`;
}

/** Core's population gate, in plain words: "it needs 2 population and the city has 1". */
const plainBlocker = (reason: string) => reason
  .replace(/^(?:\w[\w ]*? needs|Requires) population (\d+) \(currently (\d+)\)$/, 'it needs $1 population and the city has $2')
  .replace(/^[A-Z](?=[a-z])/, (c) => c.toLowerCase());

/**
 * Core reports a city's production changes it didn't choose as OrderFailed: "Hymnal can no longer
 * produce Settler: Settler needs population 2 (currently 1); its 38 Production stays banked", and
 * "Hymnal skipped queued Trooper: Needs 10 Iron …; kept in queue". These read as city news, not a
 * failed order, and say what happens to the progress and what to do next.
 */
function cityProductionFailure(reason: string): { city: string; text: string } | null {
  const stopped = /^(.+?) can no longer produce (.+?): (.+?)(?:; its (\d+) Production stays banked)?$/s.exec(reason);
  if (stopped) {
    const [, city, item, why, banked] = stopped;
    return { city, text: `${city} can no longer build ${item}: ${plainBlocker(why)}. ${banked ? `Its ${banked} Production stays banked for the ${item}. ` : ''}Choose a new build.` };
  }
  const skipped = /^(.+?) skipped queued (.+?): (.+?)(; kept in queue)?$/s.exec(reason);
  if (skipped) {
    const [, city, item, why, kept] = skipped;
    return { city, text: kept
      ? `${city}'s queued ${item} is waiting: ${plainBlocker(why).replace(/; move a blocking unit first$/, '')}. It stays in the queue and starts by itself once it can.`
      : `${city} dropped ${item} from its plan: ${plainBlocker(why)}.` };
  }
  return null;
}

/**
 * "Waypost Seven was attacked: 200 → 120 HP by 3 Soldiers": one line per own city that took
 * damage, first in the report (user, 2026-10-08: "I shouldn't be surprised to see that one of my
 * cities is about to get taken over"). The battle lines below it keep the strike-by-strike math.
 */
function siegeLines(events: Event[], ctx: ReportContext): ReportLine[] {
  const mine = (p: PlayerId) => p === ctx.me || ctx.team.includes(p);
  const captured = new Set(events.flatMap((x) => (x.type === 'CityCaptured' ? [x.city] : [])));
  return cityHits(events, mine, (id) => ctx.unitKind(id), ctx.cityHpBefore).map((hit) => {
    const name = ctx.cityName(hit.city);
    const mark: HpMark = { text: `${hit.before} → ${hit.after} HP`, tone: 'bad' };
    const force = forceText(hit.attackers.map((a) => ({ kind: a.kind, faction: ctx.playerFaction?.(a.owner) })));
    const fell = captured.has(hit.city) ? ', and it fell' : hit.after <= 0 ? ', and it is at 0 HP' : '';
    const owners = fromText(hit.attackers.map((a) => a.owner), (id) => ctx.empireOf?.(id) ?? ctx.playerName(id));
    return line('attack', `${name} was attacked: ${mark.text} by ${force}${owners ? ` from ${owners}` : ''}${fell}`, {
      at: ctx.cityPos(hit.city), from: hit.from, hp: [mark], icon: 'ui-siege', weight: 2, tone: 'bad',
      players: [...new Set([ctx.me, ...hit.attackers.map((a) => a.owner)])],
      details: [
        ...hit.strikes.map((s) => `${cap(unitName(ctx, s.unit, s.owner, s.kind))} ${s.ranged ? 'bombarded' : 'struck'} it for −${s.damage} HP.`),
        CITY_HEAL_RULE,
      ],
    });
  });
}

export function formatEvents(events: Event[], ctx: ReportContext): ReportLine[] {
  ctx = { ...ctx, events };
  const out: ReportLine[] = siegeLines(events, ctx);
  const moved = new Map<number, { idx: number; steps: number; to: Hex; owner: PlayerId }>();
  // Split counterattack: one line per defender that several units attacked, and only that line
  // says whether it was destroyed. A captured city's line is its CityCaptured line.
  const assaults = combinedAssaults(events);
  const captured = new Set(events.flatMap((x) => (x.type === 'CityCaptured' ? [x.city] : [])));
  const mine = (p: PlayerId) => p === ctx.me || ctx.team.includes(p);
  for (const e of events) {
    // A hand-over of the Heir is one line, from the new holder's gain; the old holder's loss folds into it.
    // A hand-over of the Imperial Diwan is one line, from the new holder's gain.
    // The birth line says it all: the first completion of Byzantium's goal is the same event.
    if (e.type === 'FactionGoalCompleted' && e.faction === 'Byzantium' && !heirLoss(events) && events.some((x) => x.type === 'HeirBorn' && x.owner === e.player)) continue;
    if (e.type === 'FactionGoalLost' && (e.faction === 'Byzantium' || e.faction === 'Mongols' || e.faction === 'India') && events.some((x) => x.type === 'FactionGoalCompleted' && x.faction === e.faction)) continue;
    if (e.type === 'FactionGoalLost' && events.some((x) => x.type === 'SakokuReopened' && x.player === e.player)) continue;
    // The Age line tells who entered the era first, so the first empire's own entry line folds into it.
    if (e.type === 'EraEntered' && e.first && events.some((x) => x.type === 'AgeBegan' && x.era === e.era)) continue;
    if (e.type === 'UnitDestroyed') {
      const inAssault = assaults.has(assaultKey({ kind: 'Unit', unit: e.unit, owner: e.owner, unit_kind: e.unit_kind }));
      const garrisonOfCaptured = events.some((x) => x.type === 'CityCaptured' && x.from === e.owner
        && hexEq(ctx.cityPos(x.city), e.at) && assaults.has(assaultKey({ kind: 'City', city: x.city, owner: x.from })));
      if (inAssault || garrisonOfCaptured) continue;
    }
    if (e.type === 'Battle' && !e.ranged && !e.meeting && typeof e.attacker === 'number' && assaults.has(assaultKey(e.defender))) {
      const group = assaults.get(assaultKey(e.defender))!;
      const d = e.defender;
      if (group[0] !== e || (d.kind === 'City' && captured.has(d.city))) continue;
      const hpAfter = Math.min(...group.map((b) => b.defender_hp_after));
      const force = assaultForce(ctx, group);
      const split = `its counterattack was split, ${counterSplit(group)}`;
      const text = d.kind === 'City'
        ? `${d.owner === ctx.me ? 'Your city ' : ''}${ctx.cityName(d.city)} ${hpAfter <= 0 ? 'fell to 0 HP under' : 'held against'} a combined assault by ${force}; ${split}${hpAfter > 0 ? ` (${hpAfter} HP left)` : ''}`
        : `${cap(unitLabel(ctx, d.unit, d.owner, d.unit_kind))} ${hpAfter <= 0 ? 'was destroyed' : 'held'} ${at(ctx, e.at)} ${hpAfter <= 0 ? 'in' : 'against'} a combined assault by ${force}; ${split}${hpAfter > 0 ? ` (${hpAfter} HP left)` : ''}`;
      out.push(line('combat', text, {
        at: e.at, from: e.attacker_at ?? null, players: [e.attacker_owner, d.owner], weight: 2,
        tone: mine(e.attacker_owner) && !mine(d.owner) ? 'good' : mine(d.owner) && !mine(e.attacker_owner) ? 'bad' : 'neutral',
        details: group.map((b) => `${unitLabel(ctx, b.attacker as number, b.attacker_owner)} dealt ${b.damage_to_defender} and took ${b.damage_to_attacker}.`),
        icon: d.kind === 'City' && hpAfter <= 0 ? 'emblem-captured' : d.kind === 'City' ? 'fortify' : 'unit-soldier',
      }));
      continue;
    }
    if (e.type === 'UnitMoved' && (!e.from || !e.to)) moved.delete(e.unit);
    if (e.type === 'UnitMoved' && e.from && e.to) {
      const m = moved.get(e.unit);
      if (m) {
        m.steps++;
        m.to = e.to;
        out[m.idx] = line('movement', `${unitLabel(ctx, e.unit, e.owner)} moved ${m.steps} tile${m.steps === 1 ? '' : 's'}, now ${at(ctx, e.to)}`, {
          at: e.to,
          players: [e.owner],
        });
        continue;
      }
      moved.set(e.unit, { idx: out.length, steps: 1, to: e.to, owner: e.owner });
    }
    out.push(formatEvent(e, ctx));
  }
  return out;
}

/**
 * The viewer's Settlers whose destination stopped being a place to found a city this turn (a city
 * founded nearby, a border grown over it), judged only from what the viewer knows. Core keeps the
 * move going, so without this line the Settler would arrive and wait with no explanation.
 */
export function settlerSiteLines(before: PlayerView | null, after: PlayerView, ctx: ReportContext): ReportLine[] {
  if (!before) return [];
  return after.units.flatMap((u) => {
    const dest = before.units.find((b) => b.id === u.id)?.goto;
    if (u.owner !== after.me || u.kind !== 'Settler' || !dest || foundCityBlocker(before, dest)) return [];
    const why = foundCityBlocker(after, dest);
    if (!why) return [];
    const settler = `${unitLabel(ctx, u.id, u.owner, u.kind)} ${at(ctx, u.pos)}`;
    return [hexEq(u.pos, dest)
      ? line('orders', `${settler} stopped at its destination, but no city can be founded there now: ${why}`, {
        at: dest, players: [u.owner], weight: 2, tone: 'bad', details: ['Pick another site: the Settle lens shows the best spots.'] })
      : line('orders', `${settler} is heading for a site where no city can be founded now: ${why}`, {
        at: dest, players: [u.owner], weight: 2, tone: 'bad', details: ['It keeps moving there until you give it a new destination. The Settle lens shows the best spots.'] })];
  });
}

/** Your garrison the occupation lock held, free now that its city is secure: one line per unit, until it is answered. */
export function releasedGarrisonLines(before: PlayerView | null, after: PlayerView, ctx: ReportContext): ReportLine[] {
  return releasedGarrison(before, after).flatMap(({ unit, city }) => {
    const u = after.units.find((x) => x.id === unit)!;
    return [line('orders', `${unitLabel(ctx, u.id, u.owner, u.kind)}: ${city} is secure. Needs orders.`, {
      at: u.pos, players: [u.owner], weight: 1, tone: 'good', icon: 'fortify' })];
  });
}

/**
 * Age cards you have just come close to (one short of the target, or within a tenth of it), in an Age you have not scored:
 * "Close to the Classical Age's VP: Gain 4 cities (3 of 4, 1 to go)". Only the turn it first gets that close.
 */
export function objectiveCloseLines(before: PlayerView | null, after: PlayerView): ReportLine[] {
  return closeObjectives(before, after).map(({ era, line: o }) => line('victory',
    `Close to the ${era.name} Age’s VP: ${o.text} (${o.current} of ${o.target}, ${o.remaining} to go)`, {
      players: [after.me], weight: 2, tone: 'good',
      details: [`Meet the ${era.name} Age’s card to score its 1 VP, kept for good.`],
    }));
}

export type ReportFilter = 'mine' | 'team' | 'all';

export function filterLines(lines: ReportLine[], filter: ReportFilter, ctx: ReportContext): ReportLine[] {
  if (filter === 'all') return lines;
  const set = filter === 'mine' ? [ctx.me] : ctx.team;
  // Lines with no players (global announcements) always show.
  return lines.filter((l) => !l.players.length || l.players.some((p) => set.includes(p)));
}

export function groupLines(lines: ReportLine[]): { category: EventCategory; lines: ReportLine[] }[] {
  return CATEGORY_ORDER.map((category) => ({
    category,
    lines: lines.filter((l) => l.category === category).sort((a, b) => b.weight - a.weight),
  })).filter((g) => g.lines.length);
}

/** Unit paths per unit from UnitMoved events, in order, for the slide animation. */
export function movementPaths(events: Event[]): Map<number, Hex[]> {
  const paths = new Map<number, Hex[]>();
  const obscured = new Set<number>();
  for (const e of events) {
    if (e.type !== 'UnitMoved') continue;
    if (!e.from || !e.to) {
      obscured.add(e.unit);
      paths.delete(e.unit);
      continue;
    }
    if (obscured.has(e.unit)) continue;
    const p = paths.get(e.unit);
    if (p) p.push(e.to);
    else paths.set(e.unit, [e.from, e.to]);
  }
  return paths;
}

/** How many lines are good and bad for the viewer (the category chips: "Combat 1 bad · 1 good"). */
export function toneCounts(lines: ReportLine[]): Record<Tone, number> {
  const out: Record<Tone, number> = { good: 0, bad: 0, neutral: 0 };
  for (const l of lines) out[l.tone]++;
  return out;
}

/** A line's text split around its HP marks, in order, so each signed number can be coloured. */
export function lineSegments(l: ReportLine): { text: string; tone?: Tone }[] {
  const out: { text: string; tone?: Tone }[] = [];
  let rest = l.text;
  for (const m of l.hp ?? []) {
    const i = rest.indexOf(m.text);
    if (i < 0) continue;
    if (i > 0) out.push({ text: rest.slice(0, i) });
    out.push({ text: m.text, tone: m.tone });
    rest = rest.slice(i + m.text.length);
  }
  if (rest) out.push({ text: rest });
  return out;
}

/** An item in a report line: an improvement names its tile ("Farm, 1 tile W"), anything else its own name. */
function builtName(item: ProductionItem, ctx: { cityPos(city: number): Hex | null | undefined }, city: number, faction?: Faction): string {
  const pos = item.kind === 'Improvement' ? ctx.cityPos(city) : null;
  return item.kind === 'Improvement' && pos ? improvementTitle(item, pos, null) : itemName(item, faction);
}

// A city under attack is never a surprise (user, 2026-10-08, playtest Switzerland T59: "this was
// the first turn I realized this city was even under attack"). The selection and forecast logic
// behind every surface that says so: the This Turn item, the map banner and the turn-report line.
// Public numbers only: the player's fogged view (city HP and defense, visible units, war status)
// plus last turn's visible events and `CityDetail.attacked_last_turn` (which survives a save).
import { isThreat } from './diplomacy';
import { CITY_HEAL, CITY_HP, damage, idiv, unitAttackStrength, unitDefenseStrength, unitRangedStrength } from './combat';
import { attacksInMelee, itemName, pluralUnit } from './data';
import { distance, hexEq, tileIndex } from './hex';
import { empireOfPlayer } from './empires';
import { isBarbarian } from './barbarians';
import type { Event, Faction, Hex, PlayerView, UnitKind, UnitView } from './types';

/** An enemy army this close to a city (and not yet beside it) raises the early warning. */
const APPROACH_RANGE = 3;
/** Enemies in the army needed for the early warning; one scout-sized visitor is not an army. */
const ARMY_SIZE = 2;
/** The forecast gives up beyond this many turns ("holds for now"). */
const HORIZON = 12;

/** Last turn's hits on one city, from its visible battle events. */
export interface CityHit {
  city: number;
  /** HP before the first hit and after the last, as the battle events report them. */
  before: number;
  after: number;
  lost: number;
  /** The units that struck it, once each, in order of first strike. A city's own shot is not an attacker. */
  attackers: { unit: number; owner: number; kind: UnitKind | null }[];
  /** Where the first attacker stood, for the report's arrow. */
  from: Hex | null;
  /** Hits in order: attacker unit, damage and HP left, for the line's details. */
  strikes: { unit: number; owner: number; kind: UnitKind | null; damage: number; left: number; ranged: boolean }[];
}

/**
 * Every team city that took damage in these events, in the order first hit. `mine` says whose
 * cities count (the viewer's team); `kindOf` names an attacker's unit kind when the viewer knows it;
 * `hpBefore` is the city's HP at the start of the turn when the view before it is known. One stage's
 * attacks are all aimed from the same snapshot, so every Battle of a stage reports the same HP left:
 * without the earlier view the starting HP is the HP left plus every hit (more than it really had
 * when a hit overshoots 0).
 */
export function cityHits(events: readonly Event[], mine: (owner: number) => boolean, kindOf: (unit: number) => UnitKind | null, hpBefore?: (city: number) => number | null): CityHit[] {
  const out = new Map<number, CityHit>();
  for (const e of events) {
    if (e.type !== 'Battle' || e.defender.kind !== 'City' || !mine(e.defender.owner) || mine(e.attacker_owner) || e.damage_to_defender <= 0) continue;
    if (typeof e.attacker !== 'number') continue;
    const id = e.defender.city;
    const kind = kindOf(e.attacker);
    let hit = out.get(id);
    if (!hit) {
      hit = { city: id, before: 0, after: e.defender_hp_after, lost: 0, attackers: [], from: e.attacker_at ?? null, strikes: [] };
      out.set(id, hit);
    }
    hit.after = Math.min(hit.after, e.defender_hp_after);
    hit.lost += e.damage_to_defender;
    if (!hit.attackers.some((a) => a.unit === e.attacker)) hit.attackers.push({ unit: e.attacker, owner: e.attacker_owner, kind });
    hit.strikes.push({ unit: e.attacker, owner: e.attacker_owner, kind, damage: e.damage_to_defender, left: e.defender_hp_after, ranged: e.ranged });
  }
  for (const hit of out.values()) {
    const known = hpBefore?.(hit.city) ?? null;
    hit.before = known !== null && known >= hit.after ? known : hit.after + hit.lost;
    hit.lost = hit.before - hit.after;
  }
  return [...out.values()];
}

/** The hits of the turn that just resolved, for the viewer's team, from the views either side of it. */
export function hitsForTurn(events: readonly Event[], before: PlayerView | null, after: PlayerView): CityHit[] {
  const ours = teamIds(after);
  const kinds = new Map([...(before?.units ?? []), ...after.units].map((u) => [u.id, u.kind]));
  const hp = new Map((before?.cities ?? []).map((c) => [c.id, c.hp]));
  return cityHits(events, (o) => ours.has(o), (id) => kinds.get(id) ?? null, (id) => hp.get(id) ?? null);
}

/** "the Byzantium · Rival", "the barbarians": whose units these are, in the empire names used everywhere else. */
export function fromText(owners: readonly number[], name: (id: number) => string): string {
  const names = [...new Set(owners)].map((id) => (isBarbarian(id) ? 'the barbarians' : name(id)));
  return names.length < 2 ? (names[0] ?? '') : `${names.slice(0, -1).join(', ')} and ${names.at(-1)}`;
}

const plural = (name: string, n: number) => (n === 1 ? name : pluralUnit(name));

/** "3 Soldiers", "2 Soldiers and a Ranger"; unknown kinds count as units. */
export function forceText(kinds: readonly ({ kind: UnitKind | null; faction?: Faction })[]): string {
  const counts = new Map<string, number>();
  for (const k of kinds) {
    const name = k.kind ? itemName({ kind: 'Unit', unit: k.kind }, k.faction) : 'unseen unit';
    counts.set(name, (counts.get(name) ?? 0) + 1);
  }
  const parts = [...counts].map(([name, n]) => (n === 1 ? `${/^[AEIOU]/i.test(name) ? 'an' : 'a'} ${name}` : `${n} ${plural(name, n)}`));
  return parts.length < 2 ? (parts[0] ?? '') : `${parts.slice(0, -1).join(', ')} and ${parts.at(-1)}`;
}

export type SiegeLevel = 'now' | 'soon' | 'holding' | 'quiet';

export interface Forecast {
  /** Turns until the city is taken at this pace, or null when it is not on course to fall. */
  turns: number | null;
  /** The headline: "Falls next turn unless reinforced". */
  text: string;
  /** The same in a few words for the turn's call to action: "falls next turn", "falls in about 2 turns". */
  short: string;
  /** The math behind it, in words (pillar 5). */
  detail: string;
  /** How loudly to say it: `now` falls next turn, `soon` within 3, `holding` otherwise. */
  level: SiegeLevel;
}

export interface CityAlarm {
  kind: 'attack' | 'approach';
  cityId: number;
  name: string;
  hp: number;
  maxHp: number;
  /** Last turn's hit, when the turn report is at hand (null after a load). */
  hit: CityHit | null;
  /** The city took damage last turn (from the report or the city's own record). */
  attackedLastTurn: boolean;
  /** Hostile units that can strike it next turn: beside it, or ranged within range. */
  attackers: UnitView[];
  /** The rest of the hostile army within `APPROACH_RANGE`, still on its way. */
  approaching: UnitView[];
  /** Expected damage per turn at average rolls. */
  perTurn: number;
  forecast: Forecast;
  /** What the attackers are, in words: "3 Soldiers". */
  force: string;
  /** Whose they are: "Byzantium · Rival". */
  from: string;
}

const teamIds = (view: PlayerView) => new Set(view.players.filter((p) => p.team === view.team).map((p) => p.id));
const factionOf = (view: PlayerView, owner: number): Faction | undefined => view.players.find((p) => p.id === owner)?.faction;

/** Visible military units that threaten this team: at war with it, but never barbarians for the Celts (scouts are no army). */
function hostileUnits(view: PlayerView): UnitView[] {
  const ours = teamIds(view);
  return view.units.filter((u) => u.military && u.kind !== 'Scout' && u.hp > 0 && !ours.has(u.owner) && isThreat(view, u.owner));
}

/** How a hostile unit can strike a city next turn: ranged from within its range, or melee beside it. */
function strikeMode(u: UnitView, city: { pos: UnitView['pos'] }): 'ranged' | 'melee' | null {
  const d = distance(u.pos, city.pos);
  // A mounted archer beside the city charges (it can take it); a pure shooter fires.
  if (u.ranged > 0 && d <= Math.max(1, u.range) && !(d === 1 && attacksInMelee(u.kind))) return 'ranged';
  if (d === 1 && u.strength > 0) return 'melee';
  return null;
}

function strikeStrength(view: PlayerView, u: UnitView, mode: 'ranged' | 'melee'): number {
  const faction = factionOf(view, u.owner);
  return (mode === 'ranged' ? unitRangedStrength(u.kind, u.hp, faction, u.level ?? 0) : unitAttackStrength(u.kind, u.hp, faction, u.level ?? 0)).total;
}

/**
 * The capture forecast from public numbers: each attacker's damage at an average roll against the
 * city's defense (which falls as it is wounded), turn by turn. At 0 HP an ungarrisoned city is
 * taken by the next melee attacker, a garrisoned one when the garrison falls too. Cities do not
 * heal while an enemy is adjacent.
 */
export function captureForecast(view: PlayerView, cityId: number, attackers: readonly UnitView[]): { forecast: Forecast; perTurn: number } {
  const city = view.cities.find((c) => c.id === cityId)!;
  const modes = attackers.map((u) => ({ u, mode: strikeMode(u, city) })).filter((m): m is { u: UnitView; mode: 'ranged' | 'melee' } => m.mode !== null);
  if (!modes.length) return { perTurn: 0, forecast: { turns: null, level: 'quiet', text: 'Nothing beside it now', short: 'nothing beside it', detail: `${city.name} regains ${CITY_HEAL} HP a turn while no enemy stands next to it.` } };
  const wound = (h: number) => idiv(CITY_HP - h, 20);
  const defense = (hp: number) => Math.max(1, city.strength + wound(city.hp) - wound(hp));
  const melee = modes.some((m) => m.mode === 'melee');
  const turnDamage = (hp: number) => modes.reduce((sum, m) => sum + damage(strikeStrength(view, m.u, m.mode), defense(hp), 100), 0);
  const first = turnDamage(city.hp);
  const garrison = view.units.find((u) => u.military && teamIds(view).has(u.owner) && hexEq(u.pos, city.pos)) ?? null;
  const garrisonDefense = garrison ? unitDefenseStrength(garrison.kind, garrison.hp, view.tiles[tileIndex(garrison.pos, view.width, view.height)]?.terrain ?? 'Plains', garrison.fortify, factionOf(view, garrison.owner), garrison.level ?? 0).total : 0;
  // Turn by turn: wear the city to 0 HP, then (if garrisoned) the garrison, then it is taken.
  let hp = city.hp, grHp = garrison?.hp ?? 0, zeroTurn: number | null = hp <= 0 ? 0 : null, fell: number | null = null;
  for (let t = 1; t <= HORIZON && fell === null; t++) {
    if (hp > 0) {
      hp = Math.max(0, hp - turnDamage(hp));
      if (hp <= 0) zeroTurn = t;
      if (hp <= 0 && !garrison && melee) fell = t;
    } else if (garrison) {
      grHp -= modes.reduce((sum, m) => sum + damage(strikeStrength(view, m.u, m.mode), garrisonDefense, 100), 0);
      if (grHp <= 0 && melee) fell = t;
    } else if (melee) fell = t;
  }
  const sum = `${forceText(modes.map((m) => ({ kind: m.u.kind, faction: factionOf(view, m.u.owner) })))} hit for about ${first} HP a turn at average rolls against defence ${defense(city.hp)}`;
  const guard = garrison ? ` Its ${itemName({ kind: 'Unit', unit: garrison.kind }, factionOf(view, garrison.owner))} (${garrison.hp} HP) must fall before it is taken.` : ' Nothing garrisons it.';
  const detail = `${sum}; ${city.name} has ${city.hp} HP and does not heal while they stand beside it.${guard}${melee ? '' : ' Ranged units wear it down but cannot take it.'}`;
  if (fell !== null) {
    const text = fell <= 1 ? 'Falls next turn unless reinforced' : `Holds about ${fell} turns unless reinforced`;
    return { perTurn: first, forecast: { turns: fell, text, short: fell <= 1 ? 'falls next turn' : `falls in about ${fell} turns`, detail, level: fell <= 1 ? 'now' : fell <= 3 ? 'soon' : 'holding' } };
  }
  if (!melee && zeroTurn !== null) {
    return { perTurn: first, forecast: { turns: null, short: city.hp <= 0 ? 'at 0 HP' : zeroTurn <= 1 ? '0 HP next turn' : `0 HP in ${zeroTurn} turns`, text: city.hp <= 0 ? 'At 0 HP; no melee unit to take it yet' : zeroTurn <= 1 ? 'Drops to 0 HP next turn; no melee unit to take it yet' : `Drops to 0 HP in about ${zeroTurn} turns; no melee unit to take it yet`, detail, level: zeroTurn <= 1 ? 'soon' : 'holding' } };
  }
  return { perTurn: first, forecast: { turns: null, short: 'holding for now', text: `Holds for now: it loses about ${first} HP a turn`, detail, level: 'holding' } };
}

/**
 * One alarm per team city that took damage last turn or has an enemy beside it (or in ranged
 * reach), then an early warning for each other city an enemy army is closing on. Most urgent first.
 * `lastHits` are the hits of the turn that just resolved (`hitsForTurn`; null after a load).
 */
export function citiesUnderAttack(view: PlayerView, lastHits: readonly CityHit[] | null): CityAlarm[] {
  if (view.finished || view.players.find((p) => p.id === view.me)?.alive === false) return [];
  const ours = teamIds(view);
  const hostile = hostileUnits(view);
  const hits = new Map((lastHits ?? []).map((h) => [h.city, h]));
  const out: CityAlarm[] = [];
  for (const city of view.cities) {
    if (!ours.has(city.owner)) continue;
    const attackers = hostile.filter((u) => strikeMode(u, city) !== null);
    const hit = hits.get(city.id) ?? null;
    const attackedLastTurn = !!hit || !!city.detail?.attacked_last_turn;
    const nearby = hostile.filter((u) => distance(u.pos, city.pos) <= APPROACH_RANGE && !attackers.includes(u));
    if (attackers.length || attackedLastTurn) {
      const { forecast, perTurn } = captureForecast(view, city.id, attackers);
      out.push({ kind: 'attack', cityId: city.id, name: city.name, hp: city.hp, maxHp: city.max_hp, hit, attackedLastTurn, attackers, approaching: nearby, perTurn, forecast,
        force: forceText(attackers.map((u) => ({ kind: u.kind, faction: factionOf(view, u.owner) }))),
        from: fromText(attackers.map((u) => u.owner), (id) => empireOfPlayer(view, id)) });
      continue;
    }
    // Early warning: an army (two or more of one empire) within three tiles, not yet beside the city.
    const byOwner = new Map<number, UnitView[]>();
    for (const u of nearby) byOwner.set(u.owner, [...(byOwner.get(u.owner) ?? []), u]);
    const army = [...byOwner.values()].filter((g) => g.length >= ARMY_SIZE).sort((a, b) => b.length - a.length)[0];
    if (!army) continue;
    const nearest = Math.min(...army.map((u) => distance(u.pos, city.pos)));
    out.push({ kind: 'approach', cityId: city.id, name: city.name, hp: city.hp, maxHp: city.max_hp, hit: null, attackedLastTurn: false, attackers: [], approaching: army, perTurn: 0,
      force: forceText(army.map((u) => ({ kind: u.kind, faction: factionOf(view, u.owner) }))),
      from: fromText(army.map((u) => u.owner), (id) => empireOfPlayer(view, id)),
      forecast: { turns: null, level: 'soon', short: `army ${nearest} tiles away`, text: `Reaches it ${nearest <= 2 ? 'next turn' : 'in about 2 turns'}: reinforce or prepare`,
        detail: `${forceText(army.map((u) => ({ kind: u.kind, faction: factionOf(view, u.owner) })))} stand${army.length === 1 ? 's' : ''} ${nearest} tiles from ${city.name}, ${army.length} units, at War with you.` } });
  }
  const rank = (a: CityAlarm) => (a.kind === 'attack' ? 0 : 10) + { now: 0, soon: 1, holding: 2, quiet: 3 }[a.forecast.level];
  return out.sort((a, b) => rank(a) - rank(b) || a.hp - b.hp || a.cityId - b.cityId);
}

/** Cities the map should draw alarmed: the attacked ones (an approaching army only warns in This Turn). */
export function alarmedCities(alarms: readonly CityAlarm[]): Map<number, SiegeLevel> {
  return new Map(alarms.filter((a) => a.kind === 'attack').map((a) => [a.cityId, a.forecast.level]));
}

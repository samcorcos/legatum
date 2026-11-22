// What an aircraft in a city's hangar is doing, in plain words, and whether it needs the player (user,
// 2026-10-17, Fighter at T204: "it's not clear to me that anything happened after that"). One place
// for the Air wing card, the Return queue, This Turn and the map's range rings, so they never disagree.
import { RULES, UNIT_NAME } from './data';
import { guardRange, isFighter, readyNotice, strikeRange } from './airText';
import { distance, tileIndex, within } from './hex';
import { atWar, barbariansSpareMe, warQueuedNow } from './diplomacy';
import { isBarbarian, ownerName } from './barbarians';
import { empireOfPlayer } from './empires';
import { pendingAirOrder } from './orders';
import type { AirPick, AircraftView, CityView, Hex, Order, PlayerView, TechId, UnitKind, UnitView } from './types';
import { airRings, type AirOverlay } from './map/airOverlay';

export { isFighter, strikeRange, guardRange, readyNotice };

/** `surprise`: you are at Peace but a War declared this turn opens before the strike resolves, so the strike is legal now. */
export interface Spot { at: Hex; name: string; owner: number; team: number; city: boolean; surprise: boolean }

/**
 * Visible enemy military units and cities within `range` of the city. `hostile` can be struck this turn: at
 * war already, or a War declared this turn (in `orders`) opens before the strike resolves. `peaceful` needs a
 * War declaration first.
 */
export function spotsIn(view: PlayerView, city: CityView, range: number, orders: Order[] = []): { hostile: Spot[]; peaceful: Spot[] } {
  const teamOf = (owner: number) => view.players.find((p) => p.id === owner)?.team ?? -1;
  const spots: Spot[] = [
    ...view.units.filter((u) => u.military && u.owner !== view.me && distance(u.pos, city.pos) <= range)
      .map((u) => ({ at: u.pos, owner: u.owner, team: teamOf(u.owner), city: false, surprise: false, name: `${UNIT_NAME[u.kind]} · ${ownerName(view, u.owner).split(' · ')[0]}` })),
    ...view.cities.filter((c) => c.owner !== view.me && c.visible && distance(c.pos, city.pos) <= range)
      .map((c) => ({ at: c.pos, owner: c.owner, team: teamOf(c.owner), city: true, surprise: false, name: c.name })),
  ].filter((s) => !view.players.some((p) => p.id === s.owner && p.team === view.team));
  const surprise = (s: Spot) => !atWar(view, s.owner) && warQueuedNow(view, orders, s.team);
  return {
    hostile: spots.filter((s) => atWar(view, s.owner) || surprise(s)).map((s) => ({ ...s, surprise: surprise(s) })),
    peaceful: spots.filter((s) => !atWar(view, s.owner) && !surprise(s)),
  };
}

/** The empire at war with you that could send aircraft near this city, if one is known: a city in the Flight era or later within its era's longest strike (Biplane, Bomber or Stealth Bomber) of the Fighter's cover. Rival hangars are hidden, so this is a warning, not a sighting. */
function airThreat(view: PlayerView, city: CityView): { empire: string; city: string; tiles: number } | null {
  // 1-based eras (`PlayerPublic.era`): Biplanes from Flight's era, Bombers from Air Power's, Jet Bombers from Jet Engines', Stealth Bombers from Stealth Technology's.
  const eraOf = (tech: TechId) => (view.research.catalog.find((t) => t.tech === tech)?.era ?? 0) + 1;
  const flight = eraOf('Flight'), airPower = eraOf('AirPower'), jets = eraOf('JetEngines'), stealth = eraOf('StealthTechnology');
  let best: { empire: string; city: string; tiles: number } | null = null;
  for (const c of view.cities) {
    if (c.owner === view.me || !atWar(view, c.owner) || !c.visible) continue;
    const owner = view.players.find((p) => p.id === c.owner);
    const tiles = distance(c.pos, city.pos);
    const era = owner?.era ?? 0;
    const reach = (era >= stealth ? strikeRange('StealthBomber') : era >= jets ? strikeRange('JetBomber') : era >= airPower ? strikeRange('Bomber') : strikeRange('Biplane'))
      + (view.research.known.includes('StealthTechnology') ? guardRange('StealthFighter') : guardRange('Fighter'));
    if (!owner || owner.era < flight || tiles > reach) continue;
    if (!best || tiles < best.tiles) best = { empire: empireOfPlayer(view, c.owner), city: c.name, tiles };
  }
  return best;
}

/** What the player answered for the aircraft. `planes`: kept as they are for `turn` only. `asleep`: kept until an enemy comes in reach (any turn). */
export interface Hold { turn: number; planes: number[]; asleep?: number[] }

/** Whether the player has already answered this aircraft this turn by keeping it as it is. */
const isHeld = (held: Hold | undefined, view: PlayerView, plane: number) => held?.turn === view.turn && held.planes.includes(plane);
const isAsleep = (held: Hold | undefined, plane: number) => !!held?.asleep?.includes(plane);

export type AirState =
  /** An air strike, recon, upgrade or rebase is queued. */
  | { kind: 'ordered'; order: Order }
  /** Enemies in range and nothing decided yet: the player may strike. */
  | { kind: 'ready'; targets: number }
  /** A Fighter with nothing to strike, or one the player chose to keep home: it intercepts by itself. `held`: answered (this turn, or asleep); otherwise it still needs orders. */
  | { kind: 'guard'; targets: number; held: boolean; asleep: boolean }
  /** A Bomber with no target (or one the player chose to hold). */
  | { kind: 'idle'; targets: number; held: boolean; asleep: boolean; peaceful: Spot[] };

/** The state of one aircraft this turn. `held`: the player chose "stay on guard" / "hold" for it. */
export function airState(view: PlayerView, city: CityView, plane: AircraftView, orders: Order[], held: Hold | undefined): AirState {
  const order = pendingAirOrder(orders, plane.id);
  if (order) return { kind: 'ordered', order };
  const { hostile: struck, peaceful } = spotsIn(view, city, strikeRange(plane.kind), orders);
  // Barbarians never attack the Celts, so they neither wake a sleeping plane nor ask for an answer: the Strike picker
  // still lists them (`spotsIn`), but the card does not demand a decision over them.
  const spared = barbariansSpareMe(view);
  const hostile = spared ? struck.filter((s) => !isBarbarian(s.owner)) : struck;
  // Asleep until an enemy is in reach; held for this turn only.
  const asleep = isAsleep(held, plane.id) && !hostile.length;
  const isHold = isHeld(held, view, plane.id) || asleep;
  if (hostile.length && !isHold) return { kind: 'ready', targets: hostile.length };
  return isFighter(plane.kind)
    ? { kind: 'guard', targets: hostile.length, held: isHold, asleep }
    : { kind: 'idle', targets: hostile.length, held: isHold, asleep, peaceful };
}

const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? '' : 's'}`;

/** The one line of what the aircraft is doing now, with its job in plain words. */
export function airHeadline(plane: AircraftView, city: CityView, state: AirState, view?: PlayerView): string {
  const name = city.name;
  if (isFighter(plane.kind)) {
    if (state.kind === 'ordered') return state.order.type === 'AirStrike' ? 'Striking this turn, so it will not intercept' : state.order.type === 'AirRecon' ? 'Scouting this turn; it still guards' : state.order.type === 'UpgradeAircraft' ? 'Upgrading this turn' : 'Rebasing this turn, so it will not intercept';
    if (state.kind === 'ready') return `Ready: ${plural(state.targets, 'target')} within ${strikeRange(plane.kind)} tiles`;
    return state.kind === 'guard' && state.asleep ? `Waiting on guard: covers ${guardRange(plane.kind)} tiles around ${name}` : `On guard: covers ${guardRange(plane.kind)} tiles around ${name}`;
  }
  if (state.kind === 'ordered') return state.order.type === 'AirStrike' ? 'Striking this turn' : state.order.type === 'AirRecon' ? 'Scouting this turn' : state.order.type === 'UpgradeAircraft' ? 'Upgrading this turn' : 'Rebasing this turn';
  if (state.kind === 'ready') return `Ready: ${plural(state.targets, 'target')} within ${strikeRange(plane.kind)} tiles`;
  if (state.kind === 'idle' && state.asleep) return 'Waiting: wakes when an enemy is within reach';
  if (state.kind === 'idle' && state.held) return 'Holding this turn';
  // The target list below the headline must never contradict it: at Peace the empire is in range, it just is not a target yet.
  if (state.kind === 'idle' && state.peaceful.length) return `${peacefulNames(view, state.peaceful)} in range, but you are at Peace`;
  return `No targets within ${strikeRange(plane.kind)} tiles`;
}

/** "Byzantium is" / "Byzantium and Sparta are": who is in range but not at War with you. */
function peacefulNames(view: PlayerView | undefined, peaceful: Spot[]): string {
  const names = view ? [...new Set(peaceful.map((s) => ownerName(view, s.owner)))] : ['An empire'];
  return `${names.join(' and ')} ${names.length === 1 && !/s$/.test(names[0]) ? 'is' : 'are'}`;
}

/** Why a Bomber (or a Fighter) has nothing to strike, when it is not just that nobody is there. */
export function noTargetReason(view: PlayerView, city: CityView, kind: UnitKind, orders: Order[] = []): string {
  const range = strikeRange(kind);
  const { hostile, peaceful } = spotsIn(view, city, range, orders);
  if (peaceful.length) {
    const names = [...new Set(peaceful.map((s) => ownerName(view, s.owner)))];
    return `${names.join(' and ')} ${names.length === 1 && !/s$/.test(names[0]) ? 'is' : 'are'} in range but not at War with you: declare War to strike. The strike can go in the same turn.`;
  }
  // Spared barbarians are in reach but never attack you, so the card asks for no answer over them (`airState`).
  if (hostile.length && barbariansSpareMe(view) && hostile.every((s) => isBarbarian(s.owner))) return `Only barbarians are within ${range} tiles of ${city.name}, and they don't attack you. You can still strike them.`;
  return `No enemy unit or city is visible within ${range} tiles of ${city.name}.`;
}

/** The Fighter's guard line: whether enemy air power could reach this city (if known). */
export function guardNote(view: PlayerView, city: CityView): string {
  const threat = airThreat(view, city);
  return threat
    ? `${threat.empire} (${threat.city}, ${plural(threat.tiles, 'tile')} away) is at War with you and can build Bombers.`
    : 'No empire at War with you has Bombers within reach. Enemy hangars are hidden, so this may change.';
}

/** Whether the aircraft still wants an answer this turn: nothing ordered, held or put to sleep (with a target in reach or not). */
const unanswered = (state: AirState) => state.kind === 'ready' || ((state.kind === 'guard' || state.kind === 'idle') && !state.held);

/** Your aircraft that need orders now, like any unit: no strike, rebase, hold or sleep yet (targets may be 0). */
export function aircraftNeedingOrders(view: PlayerView, orders: Order[], held: Hold | undefined): { plane: AircraftView; city: CityView; targets: number }[] {
  if (view.finished) return [];
  return (view.aircraft ?? []).filter((a) => a.owner === view.me).flatMap((plane) => {
    const city = view.cities.find((c) => c.id === plane.city && c.owner === view.me);
    if (!city) return [];
    const state = airState(view, city, plane, orders, held);
    return unanswered(state) && state.kind !== 'ordered' ? [{ plane, city, targets: state.targets }] : [];
  }).sort((a, b) => a.plane.id - b.plane.id);
}

/** Your Fighters that are on guard now (answered "stay on guard", asleep, or with no strike ordered), by city: This Turn lists them so the player knows they are active. */
export function fightersOnGuard(view: PlayerView, orders: Order[], held: Hold | undefined): { plane: AircraftView; city: CityView }[] {
  return (view.aircraft ?? []).filter((a) => a.owner === view.me && isFighter(a.kind)).flatMap((plane) => {
    const city = view.cities.find((c) => c.id === plane.city && c.owner === view.me);
    if (!city) return [];
    const state = airState(view, city, plane, orders, held);
    return state.kind === 'guard' && state.held ? [{ plane, city }] : [];
  }).sort((a, b) => a.plane.id - b.plane.id);
}

/** The hexes an open picker offers: targets within the aircraft's strike range (`peaceTeam`: needs a War declaration first), or own cities it can rebase to. */
export function pickerCandidates(view: PlayerView, city: CityView, plane: AircraftView, what: AirPick, orders: Order[] = [], errors: { index: number }[] = []): { at: Hex; city?: CityView; peaceTeam?: number }[] {
  if (what === 'strike') {
    const { hostile, peaceful } = spotsIn(view, city, strikeRange(plane.kind), orders);
    return [...hostile.map((s) => ({ at: s.at })), ...peaceful.map((s) => ({ at: s.at, peaceTeam: s.team }))];
  }
  if (what === 'recon') return reconTargets(view, city, plane.kind).map((at) => ({ at }));
  return rebaseOptions(view, city, orders, errors).filter((option) => option.available).map(({ city: c }) => ({ at: c.pos, city: c }));
}

/** Every tile on the map an aircraft can scout: 1 to its strike range from its city, seen or not (core `validate_recon`). */
export function reconTargets(view: PlayerView, city: CityView, kind: UnitKind): Hex[] {
  return within(city.pos, strikeRange(kind)).filter((at) => distance(at, city.pos) >= 1 && tileIndex(at, view.width, view.height) >= 0);
}

/** Why an aircraft cannot scout now, or null: it has used its order, or no tile is in reach. */
export function reconBlocker(view: PlayerView, city: CityView, plane: AircraftView, orders: Order[] = []): string | null {
  if (view.finished) return 'Recon unavailable: the game is over.';
  if (pendingAirOrder(orders, plane.id)) return 'Recon unavailable: this aircraft already has an order this turn (one per aircraft).';
  return reconTargets(view, city, plane.kind).length ? null : `Recon unavailable: no tile within ${strikeRange(plane.kind)} tiles of ${city.name} is on the map.`;
}

/** Every own city in rebase range, including unavailable hangars and the reason they are full. */
export function rebaseOptions(view: PlayerView, city: CityView, orders: Order[] = [], errors: { index: number }[] = []) {
  const validOrders = orders.filter((_, index) => !errors.some((error) => error.index === index));
  return view.cities
    .filter((candidate) => candidate.owner === view.me && candidate.id !== city.id)
    .map((candidate) => {
      const queued = validOrders.filter((order) => order.type === 'Rebase' && order.to === candidate.id).length;
      const count = (view.aircraft ?? []).filter((aircraft) => aircraft.city === candidate.id).length + queued;
      const cap = view.research.known.includes('StealthTechnology') ? 3 : RULES.airBaseCap;
      return { city: candidate, count, queued, cap, available: count < cap };
    });
}

/** What the map draws for the selected own city's aircraft: their range rings, and the open picker's candidates. */
export function airOverlayFor(view: PlayerView, cityId: number, air: { plane: number | null; what: AirPick | null; hover: Hex | null }, orders: Order[] = [], errors: { index: number }[] = []): AirOverlay | null {
  const city = view.cities.find((c) => c.id === cityId && c.owner === view.me);
  const planes = (view.aircraft ?? []).filter((a) => a.city === cityId && a.owner === view.me);
  if (!city || !planes.length || view.finished) return null;
  const plane = planes.find((a) => a.id === air.plane);
  const candidates = plane && air.what ? pickerCandidates(view, city, plane, air.what, orders, errors).map((c) => c.at) : [];
  return { center: city.pos, rings: airRings([...new Set(planes.map((a) => a.kind))]), candidates, pickKind: plane && air.what ? air.what : null, hover: air.hover,
    hoverLabel: plane && air.what === 'rebase' && air.hover ? view.cities.find((c) => c.owner === view.me && c.pos.q === air.hover!.q && c.pos.r === air.hover!.r)?.name ?? null : null };
}

/** A stand-in map unit for a hangar aircraft, so the unit forecast card can describe its strike (as core does for `forecast_air_strike`). */
export function aircraftAsUnit(plane: AircraftView, city: CityView): UnitView {
  const stats = RULES.unitStats[plane.kind];
  return {
    id: plane.id, owner: plane.owner, kind: plane.kind, pos: city.pos, hp: plane.hp, moves_left: 1, max_moves: 1, goto: null,
    fortify: 0, strength: stats.strength, ranged: stats.ranged, range: stats.range, military: true, level: plane.level ?? 0, upgrade_cost: null,
  };
}

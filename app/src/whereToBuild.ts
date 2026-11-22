// "Where to build this": for one item, what each of your cities would do with it. Nothing here
// computes a turn count: the turns come from the core's view, the same figures the city's own
// production list shows (`ProductionOption.turns`, which already counts banked progress), or from
// the research preview (`TechUnlock.cities`) for an item not yet researched. This module only picks
// the right figure per city, words the reasons plainly and chooses the fastest.
import { isGreatWork, itemName, sameItem } from './data';
import { productionQueue } from './orders';
import type { CityView, Order, PlayerView, ProductionItem } from './types';

/**
 * Buildings whose listed price is this much or more are worth comparing between
 * cities: the Factory and the Star Fort. world wonders always are. A display threshold, not a game rule.
 */
export const COSTLY_LISTED_COST = 300;

/**
 * ready: this city can build it now. building: it is the current build (or chosen this turn). queued: waiting in
 * the queue. elsewhere: the one allowed copy is already in another of your cities. blocked: a requirement is missing.
 * later: the technology is not researched yet; the turns are what the city would take once it is.
 */
type SiteState = 'ready' | 'building' | 'queued' | 'elsewhere' | 'blocked' | 'later';

export interface BuildSite {
  cityId: number;
  name: string;
  state: SiteState;
  /** Turns to finish here from now, counting progress already in this city. Null when it cannot be told. */
  turns: number | null;
  /** Production already in this city for the item (the active build's, or banked from a switch). */
  stored: number;
  cost: number;
  /** Plain words for a state that is not "ready" ("Needs a Library first"). */
  reason: string | null;
  /** What the city is making now, which choosing this item here would replace (its progress stays banked). */
  replaces: string | null;
  idle: boolean;
  /** The choice was made this turn and has not been played yet. */
  pending: boolean;
  /** Position in the queue (1 is first), when queued. */
  position: number | null;
}

export interface WhereToBuild {
  item: ProductionItem;
  researched: boolean;
  cost: number;
  /** The listed price before faction changes, from the core's own cost breakdown. */
  listedCost: number;
  sites: BuildSite[];
  fastest: BuildSite | null;
}

/** One per empire: a world wonder, or the Yam Relay Station. Another city already building one is the answer, not a city to compare. */
const oncePerEmpire = (item: ProductionItem): boolean => (item.kind === 'Building' || item.kind === 'Wonder') && (isGreatWork(item.building) || item.building === 'YamRelayStation');
const sameBuild = (choice: ProductionItem | null, item: ProductionItem): boolean => !!choice && (sameItem(choice, item)
  || ((item.kind === 'Wonder' || item.kind === 'Building') && oncePerEmpire(item)
    && (choice.kind === 'Wonder' || choice.kind === 'Building') && choice.building === item.building));

const a = (word: string) => `${/^[aeiou]/i.test(word) ? 'an' : 'a'} ${word}`;

/** The core's reason, in the words the build list uses ("Needs a Library first"). */
export function plainReason(reason: string): string {
  let m = /^Build in a city with (.+)$/.exec(reason);
  if (m) return `Needs ${a(m[1])} first`;
  m = /^Requires population (\d+) \(currently (\d+)\)$/.exec(reason);
  if (m) return `Needs population ${m[1]} (has ${m[2]})`;
  m = /^Requires (\d+) owned (.+) buildings \(currently (\d+)\)$/.exec(reason);
  if (m) return `Needs ${m[1]} ${m[2]} buildings (you have ${m[3]})`;
  m = /^Requires (.+)$/.exec(reason);
  if (m) return `Needs ${a(m[1])} first`;
  if (/already completed its world wonder/.test(reason)) return 'Your empire has already finished its world wonder';
  return reason;
}

/** Your own cities that have a city panel (own cities carry detail). */
const ownCities = (view: PlayerView): CityView[] => view.cities.filter((c) => c.owner === view.me && c.detail);

function siteFor(view: PlayerView, city: CityView, item: ProductionItem, orders: Order[], others: CityView[]): BuildSite {
  const d = city.detail!;
  const unlock = view.research.unlocks.find((u) => sameItem(u.item, item));
  const quote = unlock?.cities?.find((c) => c.city === city.id);
  const option = d.options.filter((o) => sameBuild(o.item, item)).sort((a, b) => a.turns - b.turns || b.production_stored - a.production_stored)[0];
  const unavailable = d.unavailable?.find((u) => sameItem(u.item, item));
  const queue = productionQueue(orders, city.id, d.producing, d.production_queue);
  const position = queue.findIndex((q) => sameBuild(q, item));
  const pending = orders.some((o) => (o.type === 'SetProduction' || o.type === 'SetProductionQueue') && o.city === city.id);
  const current = pending || d.producing ? queue[0] ?? null : null;
  const faction = view.players.find((p) => p.id === city.owner)?.faction;
  const base = {
    cityId: city.id,
    name: city.name,
    idle: !queue[0],
    pending: pending && position === 0,
    position: position >= 0 ? position + 1 : null,
    replaces: current && !sameBuild(current, item) ? itemName(current, faction) : null,
  };
  const cost = option?.cost ?? unavailable?.cost ?? unlock?.cost ?? 0;
  const known = !unlock || view.research.known.includes(unlock.tech);
  // The one world wonder per empire: if another of your cities has it, say where.
  const elsewhere = oncePerEmpire(item) && position < 0
    ? others.find((c) => c.id !== city.id && productionQueue(orders, c.id, c.detail!.producing, c.detail!.production_queue).some((q) => sameBuild(q, item)))
    : undefined;
  if (elsewhere) {
    const active = sameBuild(productionQueue(orders, elsewhere.id, elsewhere.detail!.producing, elsewhere.detail!.production_queue)[0] ?? null, item);
    // A choice made this turn keeps the turns the core quoted, so the list holds its order when you click.
    return { ...base, state: 'elsewhere', turns: option && option.turns > 0 ? option.turns : null, stored: option?.production_stored ?? 0, cost, reason: `${active ? 'Already building' : 'Already queued'} in ${elsewhere.name}; one per empire`, replaces: null, pending: false, position: null };
  }
  if (!known) {
    const blocker = quote?.blocker ? plainReason(quote.blocker) : null;
    return { ...base, state: blocker ? 'blocked' : 'later', turns: blocker ? null : quote?.build_turns ?? null, stored: 0, cost, reason: blocker, replaces: null, position: null };
  }
  if (option) {
    const active = position === 0 && current !== null && (pending ? true : sameBuild(d.producing, item));
    const turns = active && !pending ? d.turns_to_complete ?? (option.turns > 0 ? option.turns : null) : option.turns > 0 ? option.turns : null;
    return { ...base, state: active ? 'building' : position >= 0 ? 'queued' : 'ready', turns, stored: option.production_stored, cost, reason: null, replaces: active || position >= 0 ? null : base.replaces };
  }
  const why = unavailable?.reason ?? quote?.blocker ?? 'Not available in this city now';
  return { ...base, state: 'blocked', turns: null, stored: unavailable?.production_banked ?? 0, cost, reason: plainReason(why), replaces: null, position: null };
}

const CAN: SiteState[] = ['ready', 'building', 'queued', 'later'];
/** Cities with an answer come first. An "elsewhere" city that has one (the choice was made this turn) keeps its place. */
const rank = (s: BuildSite) => ((CAN.includes(s.state) || s.state === 'elsewhere') && s.turns !== null ? 0 : 1);

/** Fastest first (fewer turns, then more progress already made, then name); cities that can't build it last. */
export function sortSites(sites: BuildSite[]): BuildSite[] {
  return [...sites].sort((x, y) => rank(x) - rank(y)
    || (rank(x) === 0 ? (x.turns as number) - (y.turns as number) || y.stored - x.stored : 0)
    || x.name.localeCompare(y.name));
}

/** The city that finishes it soonest, or null when none can. */
export function fastestSite(sites: BuildSite[]): BuildSite | null {
  const best = sortSites(sites.filter((s) => CAN.includes(s.state)))[0];
  return best && rank(best) === 0 ? best : null;
}

export function whereToBuild(view: PlayerView, item: ProductionItem, orders: Order[]): WhereToBuild {
  const cities = ownCities(view);
  const sites = sortSites(cities.map((c) => siteFor(view, c, item, orders, cities)));
  const unlock = view.research.unlocks.find((u) => sameItem(u.item, item));
  return {
    item,
    researched: !unlock || view.research.known.includes(unlock.tech),
    cost: sites[0]?.cost || unlock?.cost || 0,
    listedCost: unlock?.cost_breakdown?.parts.find(([label]) => label === 'Listed cost')?.[1] ?? (sites[0]?.cost || unlock?.cost || 0),
    sites,
    fastest: fastestSite(sites),
  };
}

/**
 * Worth showing: a world wonder or a costly building, with at least two of your cities to compare. A cheap building
 * takes a few turns anywhere, and with one city there is nothing to choose.
 */
export function showsWhereToBuild(w: WhereToBuild): boolean {
  if (w.item.kind !== 'Building' || w.sites.length < 2) return false;
  return isGreatWork(w.item.building) || w.listedCost >= COSTLY_LISTED_COST;
}

export const turnsWord = (n: number) => `${n} turn${n === 1 ? '' : 's'}`;

export interface WhereSummary {
  /** "Best city for this: Bern, done in 22 turns", or why this city cannot build it yet. */
  text: string;
  kind: 'fastest' | 'building' | 'none';
}

/** The one line above the list. `here` is the city whose panel it sits in (it reads "here"), if any. */
export function whereSummary(w: WhereToBuild, here?: number): WhereSummary {
  const once = oncePerEmpire(w.item);
  const building = w.sites.find((s) => s.state === 'building' || (once && s.state === 'queued'));
  if (once && building) {
    const verb = building.state === 'building' ? 'Building' : 'Queued';
    const where = building.cityId === here ? 'here' : `in ${building.name}`;
    return { kind: 'building', text: `${verb} ${where}${building.turns ? `; ${turnsWord(building.turns)} left` : ''}` };
  }
  // A tie goes to the city whose panel it is: it is as fast as any.
  const tied = here === undefined ? undefined : w.sites.find((s) => s.cityId === here && CAN.includes(s.state) && s.turns !== null && s.turns === w.fastest?.turns);
  const best = tied ?? w.fastest;
  if (!best) {
    const reasons = [...new Set(w.sites.map((s) => s.reason).filter(Boolean))];
    return { kind: 'none', text: `No city can build it now${reasons.length === 1 ? `: ${reasons[0]!.charAt(0).toLowerCase()}${reasons[0]!.slice(1)}` : ''}` };
  }
  const mine = here === undefined ? undefined : w.sites.find((s) => s.cityId === here);
  const when = best.turns === null ? '' : `, done in ${turnsWord(best.turns)}`;
  const hereNote = mine && mine.cityId !== best.cityId
    ? CAN.includes(mine.state) && mine.turns !== null ? ` Here: ${turnsWord(mine.turns)}.`
      : mine.reason ? ` Not here yet: ${mine.reason.charAt(0).toLowerCase()}${mine.reason.slice(1)}.` : ''
    : '';
  const later = w.researched ? '' : ' once researched';
  return { kind: 'fastest', text: `Best city for this${later}: ${best.cityId === here ? 'here' : best.name}${when}.${hereNote}` };
}

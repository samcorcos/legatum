import { BUILDING_NAME, itemName, sameItem } from './data';
import { empireOfPlayer } from './empires';
import type { BuildingKind, CityDetail, CityView, Faction, GreatWorkView, PlayerView } from './types';

/**
 * A lost world-wonder race in one wording everywhere (legatum-ui, user 2026-11-07, Thun T92):
 * the rival, the wonder, and what the core did with the banked Production
 * (`rules/cities.rs::resolve_wonder_race`: half of it, rounded down, returns as Gold).
 */
export function wonderLostHeadline(rival: string, building: BuildingKind, city: string, active = true): string {
  return active
    ? `${rival} completed the ${BUILDING_NAME[building]} first: ${city} stopped building it.`
    : `${rival} completed the ${BUILDING_NAME[building]} first, so it was taken out of ${city}'s queue.`;
}

/** The refund sentence, or '' when nothing had been put into it. */
export function wonderLostRefund(production: number, gold: number, city: string): string {
  return production > 0
    ? `You got ${gold.toLocaleString('en-US')} Gold back, half the ${production.toLocaleString('en-US')} Production ${city} had put into it.`
    : '';
}

/** The headline and refund as one line (report, moment card). */
export function wonderLostLine(rival: string, building: BuildingKind, city: string, production: number, gold: number, active = true): string {
  return [wonderLostHeadline(rival, building, city, active), wonderLostRefund(production, gold, city)].filter(Boolean).join(' ');
}

/**
 * The stopped city's queue head, only when it can't start by itself: its own reason
 * (`ProductionOption.resource_blocker`). A head that can start needs no sentence: the queue line says it waits.
 */
export function blockedQueueHead(detail: CityDetail, faction?: Faction): string {
  const head = detail.production_queue?.[0];
  if (!head) return '';
  const blocker = detail.options.find((o) => sameItem(o.item, head))?.resource_blocker;
  return blocker ? `${itemName(head, faction)}, next in its queue, can't start yet: ${blocker.replace(/\.$/, '')}.` : '';
}

/** The stopped city's decision: why it has nothing to build, and the choice it wants. */
export function wonderLostDecision(view: Pick<PlayerView, 'players'>, city: Pick<CityView, 'name'>, detail: CityDetail, faction?: Faction): string | null {
  const loss = detail.wonder_lost;
  if (!loss) return null;
  const blocked = blockedQueueHead(detail, faction);
  return `${wonderLostLine(empireOfPlayer(view, loss.winner), loss.building, city.name, loss.production, loss.gold)} Choose what ${city.name} builds next.${blocked ? ` ${blocked}` : ''}`;
}

/** Turns a rival's public wonder build needs at its public rate, or null when its rate is unknown. */
export function rivalTurnsLeft(work: GreatWorkView): number | null {
  if (work.production_per_turn <= 0) return null;
  return Math.max(1, Math.ceil(Math.max(0, work.cost - work.progress) / work.production_per_turn));
}

/** Met rivals building this wonder now, nearest to finishing first (public: `projects::great_work_views`). */
export function rivalBuilders(view: Pick<PlayerView, 'players' | 'great_works' | 'me' | 'team' | 'diplomacy'>, building: BuildingKind): { name: string; turns: number | null }[] {
  return (view.great_works ?? [])
    .filter((w) => w.building === building && w.player !== view.me)
    .flatMap((w) => {
      const p = view.players.find((candidate) => candidate.id === w.player);
      if (!p || !(p.team === view.team || view.diplomacy.met?.includes(p.team))) return [];
      return [{ name: empireOfPlayer(view, w.player), turns: rivalTurnsLeft(w) }];
    })
    .sort((a, b) => (a.turns ?? Infinity) - (b.turns ?? Infinity));
}

/** "India (about 2 turns)". */
export function rivalBuilderText(r: { name: string; turns: number | null }): string {
  return r.turns === null ? r.name : `${r.name} (about ${r.turns} turn${r.turns === 1 ? '' : 's'})`;
}

/**
 * The race warning on a city building a world wonder that a met rival is also building: the
 * fastest rival's public pace against this city's own.
 */
export function wonderRaceWarning(view: Pick<PlayerView, 'players' | 'great_works' | 'me' | 'team' | 'diplomacy'>, cityName: string, building: BuildingKind, ownTurns: number | null): string | null {
  const rivals = rivalBuilders(view, building);
  if (!rivals.length) return null;
  const first = rivals[0];
  const wonder = BUILDING_NAME[building];
  const turns = (n: number) => `${n} turn${n === 1 ? '' : 's'}`;
  const others = rivals.length > 1 ? ` ${rivals.slice(1).map((r) => r.name).join(', ')} ${rivals.length > 2 ? 'are' : 'is'} building it too.` : '';
  const own = ownTurns && ownTurns > 0 ? ownTurns : null;
  // Ahead: say so first. Same turn counts as behind, since resolution order decides a tie.
  if (own !== null && first.turns !== null && own < first.turns) {
    return `${cityName} should finish the ${wonder} first (${turns(own)}; ${first.name} about ${turns(first.turns)}).${others}`;
  }
  const pace = first.turns === null ? '' : ` and should finish in about ${turns(first.turns)}`;
  return `${first.name} is also building the ${wonder}${pace}${own !== null ? `; ${cityName} needs ${own}` : ''}. If ${first.name} finishes first, ${cityName} stops and gets half its Production on it back as Gold.${others}`;
}
